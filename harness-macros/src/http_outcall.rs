use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, ItemFn, ReturnType};

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
                        // Sample: "HTTP/1.1 202\r\nContent-Type: text/plain\r\n\r\nHello, World!"
                        // trims '\r\n' from the beginning
                        let resp = &response.body[2..];
                        ::candid::Decode!(&resp, #type_path)
                        .expect("the response should implement CandidType; qed")
                    }
                },
            )
        }

        ReturnType::Default => (quote!(()), quote!()),
    };

    Ok(TokenStream::from(quote! {
        #[update]
        async fn #ident(#(#inputs),*) -> harness_primitives::HarnessResult<#output> {
            let device_url = match StateAccessor::get_next_device() {
                Ok(url) => url,
                Err(e) => return harness_primitives::HarnessResult::<#output>::wrap_error(e),
            };

            let program_id: String = StateAccessor::get_program_id().into();

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
