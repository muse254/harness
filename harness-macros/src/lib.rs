#![recursion_limit = "128"]
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::{Error, ItemFn, Signature, Type};

// `wapc_init` is reserved by the wapc protocol used in the project.
const RESERVED_METHODS: [&str; 1] = ["wapc_init"];

/// This macro is responsible for generating `harness` compatible implementations.
///
/// It only triggers when the flag `harness_impl` is used
#[proc_macro_attribute]
pub fn harness(_attr: TokenStream, item: TokenStream) -> TokenStream {
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

            create_harness_function(func).map_or_else(|e| e.to_compile_error().into(), Into::into)
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
    let (arg_types, ret_types) = get_args(&func.sig)?;

    if ret_types.len() <= 1 {
        return Err(syn::Error::new(
            Span::call_site(),
            &format!("we assume is singular or perhaps empty return type"),
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
    let harness_fn_name_const = Ident::new(&base_name.to_uppercase(), ident.span());
    let no_return = ret_types.is_empty();

    let harness_function = quote! {
        // We have this here to allow or discovery of the harness functions
        #[::linkme::distributed_slice(HARNESS_FUNCTIONS)]
        static #harness_fn_name_const: fn(&[u8]) -> ::wapc_guest::CallResult = #harness_fn_name;

        fn #harness_fn_name(payload: &[u8]) -> ::wapc_guest::CallResult {
            // TODO: allow attributes to be passed to the DecoderConfig, or pick that up from ic_cdk?
            let (#(#arg_vars,)*) = ::candid::Decode!([::candid::DecoderConfig::new()]; &payload, #(#arg_types),*)?;

            match #no_return {
                true => {
                    crate::#ident(#(#arg_vars),*);
                    Ok(vec![])
                },
                false => {
                    // if return type exists, value is of a singular type or composite type
                    let bytes =  ::candid::Encode!(crate::#ident(#(#arg_vars),*));
                    Ok(bytes.to_vec())
                }
            }
        }
    };

    Ok(TokenStream::from(quote! {
        // TODO: since were building for harness only, strip the ic_cdk annotations; leave other annotations?
        #func
        #harness_function
    }))
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
