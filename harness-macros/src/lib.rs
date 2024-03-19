//! This crate provides the `harness` and `harness_export` macros.
//!
//! To create a harness function, use the `harness` macro on a function and
//! then call `harness_export` at the end of the file to register all harness functions.
//! ```rust,ignore,no_run
//! #[harness]
//! fn hello(msg: String) -> String {
//!    format!("Hello, {msg}!")
//! }
//!
//! harness_export!();
//! ```
//!
//! Suppose we have a function that uses the ic_cdk annotations. Since `harness` is not processing the annotations,
//! we have macro syntax that can help with stripping the annotations.
//! By default, ic_cdk annotations will be stripped. Other annotations can be included by passing them as arguments:
//! ```rust,ignore,no_run
//! #[harness(strip = ["query", "other_annotation", "yet_another_annotation"])]
//! #[other_annotation]
//! #[yet_another_annotation]
//! #[ic_cdk::query]
//! fn hello(msg: String) -> String {
//!   format!("Hello, {msg}!")
//! }
//!
//! harness_export!();
//! ```
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use std::sync::Mutex;
use syn::{Error, ItemFn, Signature, Type};

// `wapc_init` is reserved by the wapc protocol used in the project.
const RESERVED_METHODS: [&str; 1] = ["wapc_init"];

// This type maps the vanilla function name to the harness function name.
// (vanilla_function_name, harness_function_name)
type FnMap = Vec<(String, String)>;

// FIXME https://github.com/rust-lang/rust/issues/44034
lazy_static::lazy_static! {
    static ref HARNESS_FUNCTIONS: Mutex<Option<FnMap>> =
    Mutex::new(Some(Vec::new()));
}

/// This macro is responsible for generating `harness` compatible implementations.
/// Any valid function compatible with `ic_cdk` annotations can be used with this macro.
///
/// It only triggers when the flag `__harness-build` is used
#[proc_macro_attribute]
pub fn harness(_attr: TokenStream, item: TokenStream) -> TokenStream {
    match syn::parse::<syn::ItemFn>(item) {
        Ok(func) => {
            if cfg!(not(feature = "__harness-build")) {
                // here to allow error discovery
                return TokenStream::from(func.to_token_stream());
            }

            if RESERVED_METHODS.iter().any(|v| func.sig.ident.eq(v)) {
                return TokenStream::from(
                    syn::Error::new(
                        Span::call_site(),
                        &format!("use of a reserved function name {}", func.sig.ident),
                    )
                    .to_compile_error(),
                );
            }

            create_harness_function(stripper(func))
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
/// It should be called after all the harness functions have been annotated with `#[harness]`.
#[must_use = "this macro should be invoked at the end of the file to register all harness functions"]
#[proc_macro]
pub fn harness_export(input: TokenStream) -> TokenStream {
    if cfg!(not(feature = "__harness-build")) {
        return input;
    }

    if !input.is_empty() {
        return syn::Error::new(Span::call_site(), "harness_export! takes no arguments")
            .to_compile_error()
            .into();
    }

    let functions = HARNESS_FUNCTIONS.lock().unwrap().clone().unwrap();
    let registration = functions
        .iter()
        .map(|(k, v)| {
            let v = syn::parse_str::<Ident>(&v).unwrap();
            quote! {
                harness_cdk::register_function(#k, #v);
            }
        })
        .collect::<Vec<_>>();

    let len = functions.len();
    let functions = functions
        .into_iter()
        .map(|(k, v)| {
            let k = k.as_str();
            let v = v.as_str();
            quote! {
                (#k, #v)
            }
        })
        .collect::<Vec<_>>();

    // TODO: cleanup this construction a bit more
    TokenStream::from(quote! {
        #[no_mangle]
        pub const HARNESS_FUNCTIONS: [(&str, &str); #len] = [#(#functions)*,];

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
            &format!("we assume a singular or perhaps empty return type"),
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

    let no_return = ret_types.is_empty();

    let arg_vars = if arg_vars.len() == 1 {
        let arg_var = &arg_vars[0];
        quote! {#arg_var}
    } else {
        quote! {#(#arg_vars),*}
    };

    let harness_impl = quote! {
        fn #harness_fn_name(payload: &[u8]) -> harness_cdk::CallResult {
            #func

            // TODO: allow attributes to be passed to the DecoderConfig, or pick that up from ic_cdk?
            let (#arg_vars) = harness_cdk::Decode!([harness_cdk::DecoderConfig::new()]; &payload, #(#arg_types),*)?;

            if #no_return {
                #ident(#arg_vars);
                return Ok(vec![]);
            }

            Ok(harness_cdk::Encode!(&#ident(#arg_vars))?)
        }
    };

    if let Some(funcs) = HARNESS_FUNCTIONS.lock().unwrap().as_mut() {
        funcs.push((
            ident.to_string(),
            format!("{}", harness_fn_name.to_token_stream()),
        ));
    }

    // TODO: since were building for harness only, strip the ic_cdk annotations; leave other annotations?
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

fn stripper(func: ItemFn) -> ItemFn {
    func
}
