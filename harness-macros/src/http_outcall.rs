use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, ItemFn, ReturnType};

pub fn impl_http_outcall(func: ItemFn) -> syn::Result<TokenStream> {
    let ident = &func.sig.ident;
    let base_name = format!("__harness_{ident}");

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

    let output = func.sig.output;
    let decode_ret = match output.clone() {
        ReturnType::Type(_, ty) => {
            let type_path = match *ty {
                syn::Type::Path(path) => path,
                _ => unreachable!("the return type is a signature, we can retrieve a path; qed"),
            };

            quote! {
                // the harness nodes speak Candid
                let payload = response.body;
                ::candid::Decode!(&payload, #type_path).expect("the response should implement CandidType; qed")
            }
        }

        ReturnType::Default => quote! {},
    };

    Ok(TokenStream::from(quote! {
        #[update]
        async fn #ident(#(#inputs),*) #output {
            let device_url =  StateAccessor::get_next_device();
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
                        value: String::from(#base_name),
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
                    if response.status !=  ::candid::Nat::from(200u8) {
                        panic!("The http_request resulted into error. \nStatus code: {}\nBody: `{:?}`", response.status, response.body);
                    }

                    #decode_ret
                }
                Err((r, m)) => {
                    panic!("The http_request resulted into error. RejectionCode: {r:?}, Error: {m}");
                }
            }
        }
    }))
}
