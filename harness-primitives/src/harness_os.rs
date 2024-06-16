//! The Harness OS is the system that manages harness programs on the device. It is responsible for loading, unloading, and executing programs.
use std::collections::HashMap;
use std::str::FromStr;

use candid::CandidType;
use wapc::WapcHost;

use crate::error::{Error, Result};

/// Holds all the harness programs that have been loaded to the device.
///
/// @muse254 See: <https://github.com/WebAssembly/wasi-threads>
/// make replications of the WasmHost & use shared memory buffer?
#[derive(Default)]
pub struct HarnessOs(HashMap<ProgramId, WapcHost>);

impl HarnessOs {
    /// This is responsible for instantiating the host process needed to load the program
    pub fn new(program_id: ProgramId, program: &[u8]) -> Result<Self> {
        let engine = wasmtime_provider::WasmtimeEngineProviderBuilder::new()
            .module_bytes(&program)
            .build()?;

        let host_instance = WapcHost::new(
            Box::new(engine),
            Some(Box::new(move |_a, _b, _c, _d, _e| Ok(vec![]))), // todo?
        )?;

        let mut harness_os = HashMap::new();
        harness_os.insert(program_id, host_instance);

        Ok(HarnessOs(harness_os))
    }

    /// Returns the list of program identifiers that are currently loaded in the device.
    pub fn program_ids(&self) -> Vec<ProgramId> {
        self.0.keys().map(|k| k.clone()).collect::<Vec<_>>()
    }

    /// This calls the operation and returns the result or appropriate errors to the caller.
    /// Note that serde to/from bytes is done inherently in the compiled program which uses candid
    pub fn call_operation(
        &self,
        program_id: &ProgramId,
        operation: &str,
        payload: &[u8],
    ) -> Result<Vec<u8>> {
        match self.0.get(program_id) {
            Some(program) => Ok(program.call(operation, payload)?),
            None => Err(Error::Internal {
                message: "the program could not be found".to_string(),
                inner: None,
            }),
        }
    }

    /// Adds a new program to the device.
    pub fn add_program(&mut self, program_id: ProgramId, program: &[u8]) -> Result<()> {
        let engine = wasmtime_provider::WasmtimeEngineProviderBuilder::new()
            .module_bytes(&program)
            .build()?;

        let host_instance = WapcHost::new(
            Box::new(engine),
            Some(Box::new(move |_a, _b, _c, _d, _e| Ok(vec![]))), // todo?
        )?;

        let mut harness_os = HashMap::new();
        harness_os.insert(program_id, host_instance);

        todo!()
    }

    /// Removes a program from the set, noop if not found.
    pub fn remove_program(&mut self, program_id: &ProgramId) {
        let _ = self.0.remove(program_id);
    }
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
    CandidType,
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
