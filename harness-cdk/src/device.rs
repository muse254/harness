use std::borrow::Cow;

use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::{storable::Bound, Storable};
use serde::Serialize;

/// Represents a device that can be registered with the arbiter
#[derive(CandidType, Deserialize, Serialize)]
pub(crate) struct Device {
    /// The unique identifier of the device
    pub uid: String,
    /// The URL of the device
    pub url: String,
    /// The public headers of the device, this can be authorization headers, etc.
    pub headers: Vec<(String, String)>,
}

impl Storable for Device {
    // TODO: bound is not used
    const BOUND: ic_stable_structures::storable::Bound = Bound::Unbounded;

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}
