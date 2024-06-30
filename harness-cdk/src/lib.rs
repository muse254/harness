//#![allow(unused_parens, unused_imports)]

pub use candid::{Decode, DecoderConfig, Encode};
pub use harness_macros::{harness, harness_export};
pub use wapc_guest::{register_function, CallResult};

mod api;
mod arbiter;
mod device;
mod utils;

mod prelude {
    pub use candid::{CandidType, Deserialize, Principal, Serialize};
    pub use ic_cdk::query;
    pub use wapc_guest::{register_function, CallResult};

    pub use crate::{arbiter::Arbiter::arbiter, device::Device::device};
    pub use harness_macros::{harness, harness_export};
}

pub use prelude::*;
