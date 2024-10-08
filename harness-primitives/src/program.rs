use std::str::FromStr;

use crate::error::Error;

/// This struct represents a program that can be loaded into the device.
pub struct Program(pub &'static [u8]);

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
pub struct ProgramId(String);

// todo!, rework
impl ProgramId {
    pub const fn new(program_id: String) -> Self {
        Self(program_id)
    }
}

impl TryFrom<String> for ProgramId {
    type Error = Error;

    fn try_from(program_id: String) -> std::result::Result<Self, Self::Error> {
        Ok(Self(program_id))
    }
}

impl FromStr for ProgramId {
    type Err = Error;

    fn from_str(program_id: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(program_id.into()))
    }
}

impl From<ProgramId> for String {
    fn from(val: ProgramId) -> Self {
        val.0
    }
}

#[test]
fn program_id_compatible_with_string() {
    use crate::program::ProgramId;
    use std::str::FromStr as _;

    // Parsing program id from str
    assert!("Using parse".parse::<ProgramId>().is_ok());
    assert!(ProgramId::from_str("Using from_str").is_ok());

    // Parsing str from program id
    let program_id = "Parsing str".parse::<ProgramId>().unwrap();
    _ = String::from(program_id);
}
