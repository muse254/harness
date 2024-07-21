// fixme: sort out this mess

// re-exports
pub use candid;
pub use harness_primitives;
pub use ic_cdk;
pub use serde_json;

mod api;
mod arbiter;
mod utils;

pub mod prelude {
    pub use candid::{self, types as candid_types, Decode, DecoderConfig, Encode, Nat};
    pub use ic_cdk::{
        self,
        api::management_canister::http_request::{
            http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse,
            TransformArgs, TransformContext,
        },
        query, update,
    };
    pub use serde_json;
    pub use wapc_guest::{register_function, CallResult};

    pub use harness_macros::{get_program, harness, harness_export};
    pub use harness_primitives;

    pub use crate::arbiter::{get_next_device, Arbiter};
}

fn scratch() {

    // let device_url = String::from("get_next_device(&NEXT_DEVICE_ID, &ARBITER)");
    // let val = ic_cdk::api::management_canister::http_request::CanisterHttpRequestArgument {
    //     url: device_url + "/procedure",
    //     max_response_bytes: None,
    // };

    // let l1 = ();
    // let l2 = ();

    // let l = vec![1,2,3,45,5];

    // let val = format!("{:?}", l.as_slice());

    // let val = candid::Encode!(l1, l2).unwrap();

    // match ic_cdk::api::management_canister::http_request::http_request({todo!()}, 10_000_000_000).await {
    //     Ok((response,)) => {
    //         // make sure the response is non-error status
    //         if response.status !=  Nat::from(200u8) {
    //             panic!("The http_request resulted into error. \nStatus code: {}\nBody: `{}`", response.status, response.body);
    //         }

    //         #decode_ret
    //     }
    //     Err((r, m)) => {
    //         panic!("The http_request resulted into error. RejectionCode: {r:?}, Error: {m}");
    //     }
    // }
}
