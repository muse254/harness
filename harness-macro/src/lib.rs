#![recursion_limit = "128"]
extern crate proc_macro;

use std::collections::BTreeMap;
use std::sync::Mutex;

use candid::types::Function as CandidFn;
use candid::types::Type as CandidTy;
use candid::Func;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::{Error, ItemFn, Signature, Type};

lazy_static::lazy_static! {
    // Carried over from `candid_derive`
    // Allows aggregation of the harness methods; which can later be registered to the
    // `wapc_init` function
    //  See https://github.com/rust-lang/rust/issues/44034
    static ref METHODS: Mutex<Option<BTreeMap<String, Function>>> =
        Mutex::new(Some(BTreeMap::default()));
}

// `wapc_init` is used by the wapc protocol used in the project.
const RESERVED_METHODS: [&str; 1] = ["wapc_init"];

// The fields still hold type information, are TokenStream raw strings
struct Function {
    args: Vec<String>,
    rets: Vec<String>,
}

/// This macro is responsible for generating `harness` compatible implementations.
///
/// It only triggers when the flag `harness_impl` is used
#[proc_macro_attribute]
pub fn harness(_: TokenStream, item: TokenStream) -> TokenStream {
    match syn::parse::<ItemFn>(item) {
        Ok(func) => {
            if cfg!(not(feature = "__harness_build")) {
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

            populate_functions(func).map_or_else(|e| e.to_compile_error().into(), Into::into)
        }
        Err(_) => TokenStream::from(
            syn::Error::new(
                Span::call_site(),
                "harness can only be used on free-standing functions.",
            )
            .to_compile_error(),
        ),
    }
}

fn create_harness_function(func: ItemFn) -> syn::Result<TokenStream> {
    let ident = &func.sig.ident;
    let harness_fn_name = { Ident::new(&format!("__harness_{ident}"), ident.span()) };

    let (args, rets) = get_args(&func.sig)?;

    let harness_function = quote! {
        fn #harness_fn_name(payload: &[u8]) -> ::wapc_guest::CallResult {
            use candid::{Decode, DecoderConfig};

            // TODO: allow attributes to be passed to the DecoderConfig, or pick that up from ic_cdk?
            let payload = Decode!([DecoderConfig::new()]; &payload, String)?;

            crate::#ident()

        }
    };

    Ok(TokenStream::from(quote! {
        #func
        #harness_function
    }))
}

fn populate_functions(func: ItemFn) -> syn::Result<TokenStream> {
    let (args, rets) = get_args(&func.sig)?;

    let args: Vec<String> = args
        .iter()
        .map(|t| format!("{}", t.to_token_stream()))
        .collect();
    let rets: Vec<String> = rets
        .iter()
        .map(|t| format!("{}", t.to_token_stream()))
        .collect();

    if let Some(map) = METHODS.lock().unwrap().as_mut() {
        map.insert(func.sig.ident.to_string(), Function { args, rets });
    }

    Ok(TokenStream::from(quote! { #func }))
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
