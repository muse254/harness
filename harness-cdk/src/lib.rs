mod arbiter;

pub mod prelude {
    pub use ic_cdk::{
        self,
        api::management_canister::http_request::{
            http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse,
            TransformArgs, TransformContext,
        },
        export_candid, query, update,
    };
    pub use serde_json;

    #[cfg(feature = "__harness-build")]
    pub use wapc_guest::{self, register_function, CallResult};

    pub use harness_macros::{get_binary__, get_schema, harness, harness_export__};
    pub use harness_primitives;

    pub use crate::arbiter::StateAccessor;
    pub use crate::harness_export;
}

/// This macro is used to initialize the arbiter with the harness program. In the case of the first build, noop.
#[macro_export]
macro_rules! harness_export {
    () => {
        #[cfg(feature = "__harness-build")]
        harness_export__!();

        // There is no security done here, research to be done on how to prevent bad actors from registering devices
        #[cfg(not(feature = "__harness-build"))]
        #[update]
        fn register_device(url: String) {
            StateAccessor::add_device(url)
        }

        // Allows the user to retrieve the program code of the harness program.
        #[cfg(not(feature = "__harness-build"))]
        #[query]
        fn get_program_code() -> Vec<u8> {
            StateAccessor::get_program_code()
        }

        // Allows the user to get the list of devices that have been registered with the arbiter.
        // there should be a way to filter if user has permissions which is not a priority at the moment.
        #[cfg(not(feature = "__harness-build"))]
        #[query]
        fn get_devices() -> Vec<String> {
            StateAccessor::get_devices()
        }

        #[cfg(not(feature = "__harness-build"))]
        #[update]
        fn remove_device(url: String) {
            StateAccessor::remove_device(url)
        }

        #[cfg(not(feature = "__harness-build"))]
        #[query]
        fn get_schema() -> harness_primitives::internals::Schema {
            StateAccessor::get_schema()
        }

        // Copied over from example `send_http_post_rust`
        // Strips all data that is not needed from the original response.
        #[cfg(not(feature = "__harness-build"))]
        #[ic_cdk::query]
        fn harness_transform(raw: TransformArgs) -> HttpResponse {
            let headers = vec![
                HttpHeader {
                    name: "Content-Security-Policy".to_string(),
                    value: "default-src 'self'".to_string(),
                },
                HttpHeader {
                    name: "Referrer-Policy".to_string(),
                    value: "strict-origin".to_string(),
                },
                HttpHeader {
                    name: "Permissions-Policy".to_string(),
                    value: "geolocation=(self)".to_string(),
                },
                HttpHeader {
                    name: "Strict-Transport-Security".to_string(),
                    value: "max-age=63072000".to_string(),
                },
                HttpHeader {
                    name: "X-Frame-Options".to_string(),
                    value: "DENY".to_string(),
                },
                HttpHeader {
                    name: "X-Content-Type-Options".to_string(),
                    value: "nosniff".to_string(),
                },
            ];

            let mut res = HttpResponse {
                status: raw.response.status.clone(),
                body: raw.response.body.clone(),
                headers,
            };

            if res.status == 200u8 {
                res.body = raw.response.body;
            } else {
                ic_cdk::api::print(format!(
                    "Received an error from harness node: err = {:?}",
                    raw
                ));
            }
            res
        }

        #[cfg(not(feature = "__harness-build"))]
        ic_cdk::export_candid!();
    };
}
