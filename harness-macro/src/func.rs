// use proc_macro::TokenStream;
// use quote::{quote, ToTokens};
// use std::collections::BTreeMap;
// use std::sync::Mutex;
// use syn::{Error, ItemFn, Result, ReturnType, Signature, Type};

// struct Method {
//     args: Vec<String>,
//     rets: Vec<String>,
// }

// lazy_static::lazy_static! {
//     // Carried over from `candid_derive`
//     // Allows aggregation of the harness methods; which can later be registered to the
//     // `wapc_init` function
//     static ref METHODS: Mutex<Option<BTreeMap<String, Method>>> =
//         Mutex::new(Some(BTreeMap::default()));
// }

// // pub(crate) fn harness(func: ItemFn) -> Result<TokenStream> {
// //     let (args, rets) = get_args(&func.sig)?;

// //     let args: Vec<String> = args
// //         .iter()
// //         .map(|t| format!("{}", t.to_token_stream()))
// //         .collect();
// //     let rets: Vec<String> = rets
// //         .iter()
// //         .map(|t| format!("{}", t.to_token_stream()))
// //         .collect();

// //     if let Some(map) = METHODS.lock().unwrap().as_mut() {
// //         map.insert(func.sig.ident.to_string(), Method { args, rets });
// //     }

// //     Ok(TokenStream::from(quote! { #fun }))
// // }
