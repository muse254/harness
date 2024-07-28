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
        ic_cdk::export_candid!();
    };
}
