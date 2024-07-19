pub use candid::{types as candid_types, Decode, DecoderConfig, Encode};

pub use harness_macros::{get_program, harness, harness_export};
pub use wapc_guest::{register_function, CallResult};

// re-exports
pub use harness_primitives;
pub use ic_cdk;
pub use serde_json;
//pub use candid;

mod api;
pub mod arbiter;
mod utils;

pub mod prelude {
    pub use harness_macros::{get_program, harness, harness_export};
    pub use candid::{types as candid_types, Decode, DecoderConfig, Encode};
}
