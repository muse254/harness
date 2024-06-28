//! This crate provides the `harness` and `harness_export` macros.

use std::{io::prelude::*, io::BufReader, sync::Mutex};

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::{Error, ItemFn, Signature, Type};

use harness_primitives::{
    ensure_path_created,
    internals::{IntermediateSchema, Schema, Service},
};

mod bootstrap;

// `wapc_init` is reserved by the wapc protocol used in the project.
const RESERVED_METHODS: [&str; 1] = ["wapc_init"];

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
/// # Examples
///
/// ```
/// use harness_macros::{harness, harness_export};
///
/// #[harness]
/// fn hello(msg: String) -> String {
///    format!("Hello, {msg}!")
/// }
///
/// harness_export!();
/// ```
///
/// We are building the application using a two-pass approach. Should refer to this [script](./) for more details.
///
/// In the first pass, we are building the harness compatible code into a wasm binary file.
/// In which case it is important to use the `--features __harness-build` flag.
///
/// In the second pass, our last pass, we are bundling the binary bytes into canister code with the relevant infrastructure.
#[proc_macro_attribute]
pub fn harness(_attr: TokenStream, item: TokenStream) -> TokenStream {
    match syn::parse::<syn::ItemFn>(item) {
        Ok(func) => {
            if cfg!(not(feature = "__harness-build")) {
                // todo: work here for the schema generation from the to upstream the schema to the `harness_cdk` @muse254

                // if HARNESS_BUILD is provided, we can now bootstrap the Arbiter code
                if bootstrap::HARNESS_BUILD.is_some() {
                    // hide the function from the second build
                    return TokenStream::from(quote!());
                }

                // here to allow error discovery
                return TokenStream::from(func.to_token_stream());
            }

            if func.sig.receiver().is_some() {
                return TokenStream::from(
                    syn::Error::new(
                        Span::call_site(),
                        "harness cannot be use on associated functions with `self` parameter",
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

            create_harness_function(func).map_or_else(|e| e.to_compile_error().into(), Into::into)
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
        if bootstrap::HARNESS_BUILD.is_some() {
            return bootstrap::harness_export_bootstrap()
                .map_or_else(|e| e.to_compile_error().into(), Into::into);
        }

        return syn::Error::new(
            Span::call_site(),
            "HARNESS_BUILD is not set, build with `--features __harness-build` and set HARNESS_BUILD to the wasm file path",
        )
        .to_compile_error()
        .into();
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
                harness_cdk::register_function(#k, #v);
            }
        })
        .collect::<Vec<_>>();

    let schema = HARNESS_SCHEMA.lock().unwrap().clone();
    let path = match ensure_path_created() {
        Ok(path) => path.to_string(),
        Err(e) => {
            return syn::Error::new(Span::call_site(), e)
                .to_compile_error()
                .into();
        }
    };

    // fixme: this assumes that at any one time, there is only one harness program being built, disambiguate for different programs
    if let Err(err) = std::fs::write(
        path + "/harness_schema.json",
        serde_json::to_string(&schema).unwrap(),
    ) {
        return syn::Error::new(Span::call_site(), err)
            .to_compile_error()
            .into();
    }

    TokenStream::from(quote! {
        #[no_mangle]
        pub fn wapc_init() {
            #(#registration)*
        }
    })
}

fn create_harness_function(func: ItemFn) -> syn::Result<TokenStream> {
    let (arg_types, ret_types) = get_args(&func.sig)?;

    // redundant check, but we want to be sure
    if ret_types.len() > 1 {
        return Err(syn::Error::new(
            Span::call_site(),
            "we assume a singular or perhaps empty return type".to_string(),
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
                let (#arg_vars) = harness_cdk::Decode!(&payload, #(#arg_types),*)?;
            }
        };

        let no_return = ret_types.is_empty();
        quote! {
            fn #harness_fn_name(payload: &[u8]) -> harness_cdk::CallResult {
                #func
                #decode_invocation
                if #no_return {
                    #fn_invocation;
                    return Ok(vec![]);
                }
                Ok(harness_cdk::Encode!(&#fn_invocation)?)
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

/// This macro allows retrieval of the compiled harness program to memory at compile time.
#[proc_macro]
pub fn get_program(_item: TokenStream) -> TokenStream {
    // get harness program compiled code
    let harness_path = match ensure_path_created() {
        Ok(path) => std::path::Path::new(path),
        Err(e) => {
            return syn::Error::new(Span::call_site(), e)
                .to_compile_error()
                .into();
        }
    };

    // fixme: assumption that the generated wasm file is at `{HARNESS_PATH}/harness_code.wasm`
    let wasm_file_path = harness_path
        .join("harness_code.wasm")
        .to_str()
        .unwrap()
        .to_string();

    // only doing fs reads at compile time
    let mut f = match std::fs::File::open(&wasm_file_path) {
        Ok(val) => val,
        Err(err) => {
            if cfg!(feature = "__harness-build") {
                return syn::Error::new(Span::call_site(), "wasm file not found, please call after the first build with `--features __harness-build`")
                    .to_compile_error()
                    .into();
            }

            return syn::Error::new(Span::call_site(), format!("wasm file not found: {}", err))
                .to_compile_error()
                .into();
        }
    };

    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)
        .expect("file read to succeed; qed");

    // fixme: how reliable is metadata for exact file size?
    let bytes = match std::fs::metadata(&wasm_file_path) {
        Ok(val) => val.len() as usize,
        Err(e) => {
            return syn::Error::new(Span::call_site(), e)
                .to_compile_error()
                .into();
        }
    };

    let schema = get_schema(&harness_path);
    TokenStream::from(quote! {
       {
        let mut buff = std::io::Cursor::new(vec![0u8; #bytes]);
        buff.read_exact(&mut vec![#(#buffer),*]).expect("buffer read to succeed; qed");

        // were sure about the size of the buffer
        let wasm = unsafe {
            &*(buff.into_inner().as_slice().as_ptr() as *const [u8; #bytes])
        };

        ::harness_primitives::program::Program {
            id: #schema.program.expect("program value expected").parse().unwrap(), // fixme: allow
            schema: #schema,
            wasm,
        }
       }
    })
}

/// Allows us to retrieve the schema generated for the harness program.
fn get_schema(harness_path: &std::path::Path) -> proc_macro2::TokenStream {
    // fixme: assumption that the schema file is at `{HARNESS_PATH}/harness_schema.json`
    let schema_file_path = harness_path
        .join("harness_schema.json")
        .to_str()
        .unwrap()
        .to_string();

    let schema_file = match std::fs::File::open(&schema_file_path) {
        Ok(val) => val,
        Err(err) => {
            if cfg!(feature = "__harness-build") {
                return syn::Error::new(Span::call_site(), "schema file not found, please call after the first build with `--features __harness-build`")
                    .to_compile_error()
                    .into();
            }

            return syn::Error::new(Span::call_site(), format!("schema file not found: {}", err))
                .to_compile_error()
                .into();
        }
    };

    let schema: Schema = serde_json::from_reader(BufReader::new(schema_file))
        .expect("schema file is not valid json");

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
            ::harness_primitives::internals::Service {
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

    quote! {
        ::harness_primitives::internals::Schema {
            program: #program,
            version: #version,
            services: vec![#(#services),*],
        }
    }
}

fn to_candid_type(ty: &mut proc_macro2::TokenStream) {
    match ty.is_empty() {
        true => {
            *ty = quote! {
                ::candid::types::internal::TypeContainer::new().add::<()>().to_string()
            }
        }
        false => {
            *ty = {
                // parse ty as a syn::Type
                let _ty = syn::parse2::<Type>(ty.clone()).expect("failed to parse type");
                quote! {
                    ::candid::types::internal::TypeContainer::new().add::<#ty>().to_string()
                }
            }
        }
    }
}
