//! This crate provides the `harness` and `harness_export` macros.
use std::io::prelude::*;
use std::sync::Mutex;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::{Error, ItemFn, Signature, Type};

use harness_primitives::{
    internals::{IntermediateSchema, Schema, Service},
    HARNESS_PATH,
};

mod http_outcall;

// `wapc_init` is reserved by the wapc protocol used in the project.
const RESERVED_METHODS: [&str; 2] = ["wapc_init", "register_function"];

// This type maps the vanilla function name to the harness function name.
// (vanilla_function_name, harness_function_name)
type FnMap = Vec<(String, String)>;

// FIXME https://github.com/rust-lang/rust/issues/44034
lazy_static::lazy_static! {
    static ref HARNESS_FUNCTIONS: Mutex<Option<FnMap>> =
    Mutex::new(Some(Vec::new()));
    static ref HARNESS_SCHEMA: Mutex<Schema> = Mutex::new(Schema::new());
}

/// This macro is responsible for generating `harness` compatible implementations.
/// Any valid function compatible with `ic_cdk` annotations can be used with this macro
/// because they serde to candid types. That is to say, the i/o types of the functions can be
/// serialized as candid types.
///
/// To create a harness function, use the `harness` macro on a function and
/// then call `harness_export` at the end of the file to register all harness functions.
///
/// # Example
///
/// ```
/// use harness_cdk::prelude::*;
///
/// #[harness]
/// fn hello(msg: String) -> String {
///    format!("Hello, {msg}!")
/// }
///
/// harness_export!();
/// ```
/// We are building the application using a two-pass approach. Should refer to this [script](./) for more details.
///
/// In the first pass, we are building the harness compatible code into a wasm binary file.
/// In which case it is important to use the `__harness-build` feature flag.
///
/// In the second pass, our last pass, we are bundling the binary bytes into canister code with the relevant infrastructure.
#[proc_macro_attribute]
pub fn harness(_attr: TokenStream, item: TokenStream) -> TokenStream {
    match syn::parse::<syn::ItemFn>(item) {
        Ok(func) => {
            if func.sig.receiver().is_some() {
                return TokenStream::from(
                    syn::Error::new(
                        Span::call_site(),
                        "harness cannot be used on associated functions with `self` parameter",
                    )
                    .to_compile_error(),
                );
            }

            if RESERVED_METHODS.iter().any(|v| func.sig.ident.eq(v)) {
                return TokenStream::from(
                    syn::Error::new(
                        Span::call_site(),
                        format!("use of a reserved function name {}", func.sig.ident),
                    )
                    .to_compile_error(),
                );
            }

            let (arg_types, ret_types) = get_args(&func.sig).unwrap();
            if cfg!(not(feature = "__harness-build")) {
                // populate the schema
                if let Err(e) = create_fn_schema_entry(&func.sig.ident, &arg_types, &ret_types) {
                    return e.to_compile_error().into();
                }

                // create the http methods for the canister
                return http_outcall::impl_http_outcall(func)
                    .map_or_else(|e| e.to_compile_error().into(), Into::into);
            }

            create_harness_function(func, &arg_types, &ret_types)
                .map_or_else(|e| e.to_compile_error().into(), Into::into)
        }
        Err(_) => TokenStream::from(
            syn::Error::new(
                Span::call_site(),
                "harness can only be used on free-standing functions",
            )
            .to_compile_error(),
        ),
    }
}

/// This macro is responsible for generating the `wapc_init` function that registers all the harness functions.
/// It should be called after all the harness functions have been annotated with `#[harness]` and brought into scope.
#[must_use = "this macro must be invoked at the end of the file to register all harness functions"]
#[proc_macro]
pub fn harness_export(input: TokenStream) -> TokenStream {
    if cfg!(not(feature = "__harness-build")) {
        let harness_code = get_binary();

        return TokenStream::from(quote! {
            use std::cell::{RefCell, Cell};

            thread_local! {
                static NEXT_DEVICE_ID: Cell<usize> = Cell::new(0); // rudimentary round robin scheduling
                static ARBITER: RefCell<Arbiter> = RefCell::new(Arbiter::new(#harness_code));
            }

            /// There is no security done here, research to be done on how to prevent bad actors from registering devices
            #[update]
            fn register_device(url: String) {
                ARBITER.with(|arbiter| {
                    arbiter.borrow_mut().add_device(url);
                });
            }

            /// Allows the user to retrieve the program code of the harness program.
            #[query]
            fn get_program_code() -> Vec<u8> {
                ARBITER.with(|arbiter| {
                    arbiter.borrow().get_program_code().expect("program code should be present"),
                }).to_vec()
            }
        });
    }

    if !input.is_empty() {
        return syn::Error::new(Span::call_site(), "harness_export! takes no arguments")
            .to_compile_error()
            .into();
    }

    let registration = HARNESS_FUNCTIONS
        .lock()
        .unwrap()
        .clone()
        .unwrap()
        .iter()
        .map(|(k, v)| {
            let v = syn::parse_str::<Ident>(v).unwrap();
            quote! {
                register_function(#k, #v);
            }
        })
        .collect::<Vec<_>>();

    TokenStream::from(quote! {
        #[no_mangle]
        pub fn wapc_init() {
            #(#registration)*
        }
    })
}

fn create_fn_schema_entry(
    ident: &Ident,
    arg_types: &[Type],
    ret_types: &[Type],
) -> syn::Result<()> {
    match HARNESS_SCHEMA.lock() {
        Ok(mut schema) => {
            let args = arg_types.iter().map(|t| quote!(#t).to_string()).collect();
            let rets = ret_types.iter().map(|t| quote!(#t).to_string()).collect();
            schema.services.push(Service {
                name: ident.to_string(),
                args,
                rets,
            });

            Ok(())
        }
        Err(e) => Err(syn::Error::new(Span::call_site(), e)),
    }
}

fn create_harness_function(
    func: ItemFn,
    arg_types: &[Type],
    ret_types: &[Type],
) -> syn::Result<TokenStream> {
    // redundant check, but we want to be sure
    if ret_types.len() > 1 {
        return Err(syn::Error::new(
            Span::call_site(),
            "we assume a singular or perhaps empty return type",
        ));
    }

    let arg_vars = arg_types
        .iter()
        .enumerate()
        .map(|(n, _)| Ident::new(&format!("__{n}_harness_var"), Span::call_site()))
        .collect::<Vec<Ident>>();

    let ident = &func.sig.ident;
    let base_name = format!("__harness_{ident}");
    let harness_fn_name = Ident::new(&base_name, ident.span());

    // isolating related TokenStream variables
    let harness_impl = {
        let arg_vars = if arg_vars.len() == 1 {
            let arg_var = &arg_vars[0];
            quote! {#arg_var}
        } else {
            quote! {#(#arg_vars),*}
        };

        let fn_invocation = if arg_types.is_empty() {
            quote! {#ident()}
        } else {
            quote! {#ident(#arg_vars)}
        };

        let decode_invocation = if arg_types.is_empty() {
            quote! {}
        } else {
            quote! {
                // TODO: allow attributes to be passed to the DecoderConfig, or pick that up from ic_cdk?
                let (#arg_vars) = Decode!(&payload, #(#arg_types),*)?;
            }
        };

        let no_return = ret_types.is_empty();
        quote! {
            fn #harness_fn_name(payload: &[u8]) -> CallResult {
                #func
                #decode_invocation
                if #no_return {
                    #fn_invocation;
                    return Ok(vec![]);
                }
                Ok(Encode!(&#fn_invocation)?)
            }
        }
    };

    if let Some(functions) = HARNESS_FUNCTIONS.lock().unwrap().as_mut() {
        functions.push((
            ident.to_string(),
            format!("{}", harness_fn_name.to_token_stream()),
        ));
    }

    if let Ok(mut schema) = HARNESS_SCHEMA.lock() {
        let args = arg_types.iter().map(|t| quote!(#t).to_string()).collect();
        let rets = ret_types.iter().map(|t| quote!(#t).to_string()).collect();
        schema.services.push(Service {
            name: base_name,
            args,
            rets,
        });
    }

    Ok(TokenStream::from(harness_impl))
}

// Carried over from `candid_derive`
fn get_args(sig: &Signature) -> syn::Result<(Vec<Type>, Vec<Type>)> {
    let mut args = Vec::new();
    for arg in &sig.inputs {
        match arg {
            syn::FnArg::Receiver(r) => {
                if r.reference.is_none() {
                    return Err(Error::new_spanned(arg, "only works for borrowed self"));
                }
            }
            syn::FnArg::Typed(syn::PatType { ty, .. }) => args.push(ty.as_ref().clone()),
        }
    }
    let rets = match &sig.output {
        syn::ReturnType::Default => Vec::new(),
        syn::ReturnType::Type(_, ty) => match ty.as_ref() {
            Type::Tuple(tuple) => tuple.elems.iter().cloned().collect(),
            _ => vec![ty.as_ref().clone()],
        },
    };

    Ok((args, rets))
}

/// This macro allows retrieval of the compiled harness program to memory at compile time. It looks into the `./harness_assets/harness_code.wasm`
/// file and reads the bytes into memory.
#[proc_macro]
pub fn get_program(_item: TokenStream) -> TokenStream {
    let schema = HARNESS_SCHEMA
        .lock()
        .expect("schema has default values")
        .clone();

    let inter_schema = IntermediateSchema::from(schema);
    let mut services = Vec::new();
    for mut service in inter_schema.services {
        service.args.iter_mut().for_each(|arg| {
            to_candid_type(arg);
        });
        to_candid_type(&mut service.rets);

        let name = service.name;
        let rets = service.rets;
        let args = service.args;
        services.push(quote! {
            harness_primitives::internals::Service {
                name: String::from(#name),
                args: vec![#(#args),*],
                rets: #rets,
            }
        });
    }

    let version = {
        if let Some(val) = inter_schema.version {
            quote!(Some(String::from(#val)))
        } else {
            quote!(None)
        }
    };

    let program = {
        if let Some(val) = inter_schema.program {
            quote!(Some(String::from(#val)))
        } else {
            quote!(None)
        }
    };

    let schema = quote! {
        harness_primitives::internals::Schema {
            program: #program,
            version: #version,
            services: vec![#(#services),*],
        }
    };

    return TokenStream::from(quote! {
        harness_primitives::program::Program {
            id: #schema.program.expect("program has default value").parse().unwrap(),
            schema: #schema,
            wasm: None,
        }
    });
}

fn get_binary() -> proc_macro2::TokenStream {
    // get harness compiled code
    let path = std::path::Path::new(HARNESS_PATH);
    let mut f = match std::fs::File::open(&path) {
        Ok(val) => val,
        Err(_) => {
            if cfg!(not(feature = "__harness-build")) {
                return syn::Error::new(Span::call_site(), format!("wasm file not found, please call after the first build with `--features __harness-build`"))
                 .to_compile_error()
                 .into();
            }

            return quote!(None);
        }
    };

    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)
        .expect("file read to succeed; qed");

    quote! { &[#(#buffer),*] }
}

fn to_candid_type(ty: &mut proc_macro2::TokenStream) {
    match ty.is_empty() {
        true => {
            *ty = quote! {
                candid_types::internal::TypeContainer::new().add::<()>().to_string()
            }
        }
        false => {
            *ty = {
                // parse ty as a syn::Type
                let _ty = syn::parse2::<Type>(ty.clone()).expect("failed to parse type");
                quote! {
                    candid_types::internal::TypeContainer::new().add::<#ty>().to_string()
                }
            }
        }
    }
}
