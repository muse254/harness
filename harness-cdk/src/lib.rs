#![cfg(feature = "__harness-build")]

use linkme::distributed_slice;

pub use harness_macro;

#[distributed_slice]
static HARNESS_FUNCTIONS: [(&str, fn(&[u8]) -> wapc_guest::CallResult)];

type HarnessFn = fn(&[u8]) -> wapc_guest::CallResult;

/// This macro should be invoked to initialize the harness code
macro_rules! harness_init {
    () => {
        #[no_mangle]
        pub fn wapc_init() {
            for (name, func) in HARNESS_FUNCTIONS {
                wapc_guest::register_function(name, *func)
            }
        }
    };
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use crate::HARNESS_FUNCTIONS;
    use candid::{Decode, DecoderConfig, Encode};
    use wapc_guest;

    #[test]
    fn candid_serde() {
        let original_val = (1u8, "One".to_string());

        // encode value
        let val = Encode!(&original_val).unwrap();
        // decode value
        let config = DecoderConfig::new();
        let val = Decode!([config]; &val, (u8, String)).unwrap();

        assert_eq!(original_val, val);
    }
}
