mod arbiter;

pub mod prelude {
    pub use ic_cdk::{
        self,
        api::management_canister::http_request::{
            http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse,
            TransformArgs, TransformContext,
        },
        query, update,
    };
    pub use serde_json;
    pub use wapc_guest::{self, register_function, CallResult, };

    pub use harness_macros::{get_binary__, get_schema, harness, harness_export__};
    pub use harness_primitives;

    pub use crate::harness_export;
    pub use crate::arbiter::StateAccessor;
}

/// This macro is used to initialize the arbiter with the harness program. In the case of the first build, noop.
#[macro_export]
macro_rules! harness_export {
    () => {
        #[cfg(feature = "__harness-build")]
        harness_export__!();

        // There is no security done here, research to be done on how to prevent bad actors from registering devices
        #[update]
        fn register_device(url: String) {
            StateAccessor::add_device(url)
        }

        // Allows the user to retrieve the program code of the harness program.
        // #[query]
        // fn get_program_code() -> Vec<u8> {
        //     ARBITER.with(|arbiter| {
        //         arbiter.borrow().get_program_code().expect("program code should be present"),
        //     }).to_vec()
        // }
    };
}
