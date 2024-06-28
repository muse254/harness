use std::str::FromStr;

use crate::error::Error;

/// This struct represents a program that can be loaded into the device.
pub struct Program {
    pub id: ProgramId,
    pub wasm: &'static [u8],
    pub schema: crate::internals::Schema,
}

/// The program identifier. It should be a human-readable identifier on the Harness network.
/// TODO: parse? `<network>.<account_id>.<program_name>`
#[derive(
    Eq,
    Ord,
    Hash,
    Clone,
    Debug,
    PartialEq,
    PartialOrd,
    candid::CandidType,
    candid::Deserialize,
    serde::Serialize,
)]
pub struct ProgramId(Box<str>);

impl TryFrom<String> for ProgramId {
    type Error = Error;

    fn try_from(program_id: String) -> std::result::Result<Self, Self::Error> {
        Ok(Self(program_id.into_boxed_str()))
    }
}

impl FromStr for ProgramId {
    type Err = Error;

    fn from_str(program_id: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(program_id.into()))
    }
}
