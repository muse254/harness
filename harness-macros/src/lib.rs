//! This crate provides the `harness` and `harness_export` macros.
use std::io::prelude::*;
use std::sync::Mutex;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::{Error, ItemFn, Signature, Type};

use harness_primitives::HARNESS_PATH;

mod http_outcall;

/// Reserved method names that cannot be used as harness functions.
const RESERVED_METHODS: [&str; 5] = [
    // `wapc_init` is reserved by the wapc protocol used in the project.
    "wapc_init",
    // `register_function` is public API for registering harness nodes.
    "register_function",
    // `get_program_code` is a public API for getting the embedded harness program.
    "get_program_code",
    // `get_devices` is a public API for getting the list of devices registered with the arbiter.
    "get_devices",
    // `remove_device` is a public API for registering devices to the arbiter.
    "register_device",
];

// This type maps the vanilla function name to the harness function name.
// (vanilla_function_name, harness_function_name)
type FnMap = Vec<(String, String)>;

// FIXME https://github.com/rust-lang/rust/issues/44034
lazy_static::lazy_static! {
    static ref HARNESS_FUNCTIONS: Mutex<Option<FnMap>> =  Mutex::new(Some(Vec::new()));
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
/// ``` ignore
/// use candid::{Encode, Decode};
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
                if let Err(e) =
                    http_outcall::create_fn_schema_entry(&func.sig.ident, &arg_types, &ret_types)
                {
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
pub fn harness_export__(input: TokenStream) -> TokenStream {
    if cfg!(not(feature = "__harness-build")) {
        // creating the schema method, all service methods are now registered in the schema
        return http_outcall::create_schema_query(input);
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
                let (#arg_vars) = ::candid::Decode!(&payload, #(#arg_types),*)?;
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
                Ok(::candid::Encode!(&#fn_invocation)?)
            }
        }
    };

    if let Some(functions) = HARNESS_FUNCTIONS.lock().unwrap().as_mut() {
        functions.push((
            ident.to_string(),
            format!("{}", harness_fn_name.to_token_stream()),
        ));
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

#[proc_macro]
pub fn get_binary__(_item: TokenStream) -> TokenStream {
    // get harness compiled code
    let path = std::path::Path::new(HARNESS_PATH);
    let mut f = match std::fs::File::open(path) {
        Ok(val) => val,
        Err(_) => {
            if cfg!(not(feature = "__harness-build")) {
                return syn::Error::new(Span::call_site(), "wasm file not found, please call after the first build with `--features __harness-build`")
                  .to_compile_error()
                  .into();
            }

            return TokenStream::from(quote!(&[]));
        }
    };

    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)
        .expect("file read to succeed; qed");

    TokenStream::from(quote! { &[#(#buffer),*] })
}
