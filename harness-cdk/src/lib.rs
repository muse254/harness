#[macro_export]
/// This macro should be called when all functions have been annotated with the #[harness] macro
macro_rules! harness_init {
    () => {
        #[no_mangle]
        pub fn wapc_init() {
            wapc_guest::register_function("hello", sample_wapc);
        }
    };
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use candid::{Decode, DecoderConfig, Encode};

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
