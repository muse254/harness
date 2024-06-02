//! This module contains the definition of the Device struct, which represents a device that can be registered with the arbiter.
//! A device should speak HTTP and be able to receive and respond to requests.

use std::borrow::Cow;

use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_stable_structures::{storable::Bound, Storable};
use serde::Serialize;

/// Represents a device that can be registered with the arbiter
#[derive(CandidType, Deserialize, Serialize)]
pub(crate) struct Device {
    /// The unique identifier of the device
    pub id: Principal,
    /// The URL of the device
    pub url: String,
    /// The programs that have been loaded to the device
    pub programs: Vec<String>,
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
