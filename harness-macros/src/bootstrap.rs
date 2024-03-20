use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, ItemFn, Result, Signature, Type};

pub(crate) const HARNESS_BUILD: Option<&'static str> = option_env!("HARNESS_BUILD");

pub(crate) fn bootstrap(item: ItemFn) -> Result<TokenStream> {
    let attributes = item
        .attrs
        .iter()
        .fold(proc_macro2::TokenStream::new(), |acc, value| {
            quote! {
               #acc
               #value
            }
        });

    let bootstrap_code = {
        let ident = item.sig.ident;
        let inputs = item.sig.inputs;
        let output = item.sig.output;
        let body = item.block;
        let vis = item.vis;
        quote! {
            #attributes
            #vis fn #ident(#inputs) #output {
                #body
            }
        }
    };

    Ok(TokenStream::from(bootstrap_code))
}

// FIXME: requires invocation of `[harness_export]` which is not stable
pub(crate) fn harness_export_bootstrap() -> Result<TokenStream> {
    let wasm_bytes = std::fs::read(
        HARNESS_BUILD.expect("`bootstrap` caller already made sure HARNESS_BUILD is set"),
    )
    .map_err(|e| {
        Error::new(
            proc_macro2::Span::call_site(),
            format!("issue reading wasm file, {e}"),
        )
    })?;

    Ok(TokenStream::from(quote! {
       const HARNESS_WASM: &[u8] = [#(#wasm_bytes)*,];
    }))
}

// Carried over from `candid_derive`
pub(crate) fn get_args(sig: &Signature) -> syn::Result<(Vec<Type>, Vec<Type>)> {
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
