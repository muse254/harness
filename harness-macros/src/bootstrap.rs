use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, Result};

pub(crate) const HARNESS_BUILD: Option<&'static str> = option_env!("HARNESS_BUILD");

// FIXME: requires invocation of `[harness_export!()]` which is not stable
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

    // let len = functions.len();
    // let functions = functions
    //     .iter()
    //     .map(|(k, v)| {
    //         let k = k.as_str();
    //         let v = v.as_str();
    //         quote! {
    //             (#k, #v)
    //         }
    //     })
    //     .collect::<Vec<_>>();

    Ok(TokenStream::from(quote! {
    //    const HARNESS_FUNCTIONS: [(&str, &str); #len] = [#(#functions)*,];
       const HARNESS_WASM: &[u8] = [#(#wasm_bytes)*,];
    }))
}
