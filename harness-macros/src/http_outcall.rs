use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use std::sync::Mutex;
use syn::{Error, ItemFn, ReturnType, Type};

use harness_primitives::internals::{IntermediateSchema, Schema, Service};

lazy_static::lazy_static! {
    static ref HARNESS_SCHEMA: Mutex<Schema> = Mutex::new(Schema::default());
}

pub fn impl_http_outcall(func: ItemFn) -> syn::Result<TokenStream> {
    let ident = &func.sig.ident;
    let procedure = ident.to_string();

    let mut inputs = Vec::new();
    let mut args = Vec::new();
    for input in &func.sig.inputs {
        match input {
            syn::FnArg::Receiver(r) => {
                if r.reference.is_none() {
                    return Err(Error::new_spanned(input, "only works for borrowed self"));
                }
            }
            syn::FnArg::Typed(syn::PatType { ty, pat, .. }) => match pat.as_ref() {
                syn::Pat::Ident(ident) => {
                    inputs.push(quote! { #ident: #ty });
                    args.push(quote! { #ident });
                }
                _ => {
                    return Err(Error::new_spanned(pat, "only works for named arguments"));
                }
            },
        }
    }

    let (output, decode_ret) = match func.sig.output.clone() {
        ReturnType::Type(_, ty) => {
            let type_path = match *ty {
                syn::Type::Path(path) => path,
                _ => unreachable!("the return type is a signature, we can retrieve a path; qed"),
            };

            (
                quote!(#type_path),
                quote! {
                    {
                        let resp = &response.body[2..];
                        ::candid::Decode!(&resp, #type_path)
                        .expect("the response should implement CandidType; qed")
                    }
                },
            )
        }

        ReturnType::Default => (quote!(()), quote!()),
    };

    let program_name = {
        let val = std::env::var("CARGO_PKG_NAME").expect("expected CARGO_PKG_NAME to be set; qed");
        quote!(String::from(#val))
    };

    Ok(TokenStream::from(quote! {
        #[update]
        async fn #ident(#(#inputs),*) -> harness_primitives::HarnessResult<#output> {
            let device_url = match StateAccessor::get_next_device() {
                Ok(url) => url,
                Err(e) => return harness_primitives::HarnessResult::<#output>::wrap_error(e),
            };

            let program_id: String = #program_name;

            // TODO: research and tweak the context for maximal cost efficiency
            let context = harness_primitives::http::Context {
                bucket_start_time_index: 0,
                closing_price_index: 4,
            };

            let body = Some(::candid::Encode!(&#(#args),*).expect("the data types should impl CandidType; qed"));

            let request = CanisterHttpRequestArgument {
                url: device_url + "/procedure",
                max_response_bytes: None,
                method: HttpMethod::POST,
                headers: vec![
                    HttpHeader {
                        name: harness_primitives::http::Header::ProgramId.to_string(),
                        value: program_id,
                    },
                    HttpHeader {
                        name: harness_primitives::http::Header::ProgramProc.to_string(),
                        value: String::from(#procedure),
                    },
                ],
                body,
                transform: Some(TransformContext::from_name(
                    "harness_transform".to_string(),
                    serde_json::to_vec(&context).unwrap(),
                )),
            };

            // TODO: This call requires cycles payment. The required cycles is a function of the request size and max_response_bytes.
            // Check [Gas and cycles cost](https://internetcomputer.org/docs/current/developer-docs/gas-cost) for more details.
            match http_request(request, 10_000_000_000).await {
                Ok((response,)) => {
                    // make sure the response is ok status
                    if response.status != ::candid::Nat::from(200u8) {
                        let body = serde_json::to_string(&response.body).unwrap_or(String::from_utf8_lossy(&response.body).to_string());
                        return harness_primitives::HarnessResult::<#output>::wrap_error_str(&format!("The http_request resulted into error. \nStatus code: {}\nBody: `{}`",
                            response.status, body));
                    }

                    harness_primitives::HarnessResult::wrap_success(#decode_ret)
                }
                Err((r, m)) => {
                    harness_primitives::HarnessResult::<#output>::wrap_error_str(&format!("The http_request resulted into error. RejectionCode: {r:?}, Error: {m}"))
                }
            }
        }
    }))
}

pub(crate) fn create_fn_schema_entry(
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

pub(crate) fn create_schema_query(_: TokenStream) -> TokenStream {
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
        let val =
            std::env::var("CARGO_PKG_VERSION").expect("expected CARGO_PKG_VERSION to be set; qed");
        quote!(String::from(#val))
    };

    let program = {
        let val = std::env::var("CARGO_PKG_NAME").expect("expected CARGO_PKG_NAME to be set; qed");
        quote!(String::from(#val))
    };

    TokenStream::from(quote! {
        #[query]
        fn get_schema() -> harness_primitives::internals::Schema {
            harness_primitives::internals::Schema {
                program: #program,
                version: #version,
                services: vec![#(#services),*],
            }
        }

        #[query]
        fn get_program_id() -> harness_primitives::program::ProgramId {
            get_schema().program.parse::<harness_primitives::program::ProgramId>().unwrap()
        }
    })
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
