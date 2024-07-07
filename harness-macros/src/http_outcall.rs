use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, ReturnType};

pub(crate) fn impl_http_outcall(func: ItemFn) -> syn::Result<TokenStream> {
    let ident = &func.sig.ident;
    let base_name = format!("__harness_{ident}");
    let inputs = func.sig.inputs;
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
                harness_cdk::Decode!(payload, #type_path).expect("the response should implement CandidType; qed")
            }
        }

        ReturnType::Default => quote! {},
    };

    Ok(TokenStream::from(quote! {
        use harness_cdk::{ic_cdk, candid, harness_primitives, serde_json};

        #[ic_cdk::update]
        async fn #ident(#inputs) #output {
            let device_url = ::arbiter::get_next_device(&NEXT_DEVICE_ID, &ARBITER);
            let program_id: String = ARBITER.with(|arbiter| arbiter.borrow().get_program_id()).into();

            // TODO: research and tweak the context for maximal cost efficiency
            let context = ::harness_primitives::http::Context {
                bucket_start_time_index: 0,
                closing_price_index: 4,
            };

            let body = Some(harness_cdk::Encode!(#inputs).expect("the data types should impl CandidType"));

            let request = CanisterHttpRequestArgument {
                url: device_url.to_string() + "/procedure",
                max_response_bytes: None,
                method: HttpMethod::POST,
                headers: vec![
                    HttpHeader {
                        name: harness_primitives::http::Header::ProgramId.to_string(),
                        value: program_id,
                    },
                    HttpHeader {
                        name: harness_primitives::http::Header::Procedure.to_string(),
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
                    // make sure the response is non-error status
                    if response.status_code !=  candid::Nat::from(200u8) {
                        panic!("The http_request resulted into error. \nStatus code: {}\nBody: `{}`", response.status_code, response.body);
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
