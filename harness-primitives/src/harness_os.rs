#![cfg(feature = "wasm-ext")]
//! The Harness OS is the system that manages harness programs on the device. It is responsible for loading, unloading, and executing programs.
use std::collections::HashMap;

use wapc::WapcHostAsync;

use crate::error::{Error, Result};
use crate::program::ProgramId;

/// Holds all the harness programs that have been loaded to the device.
#[derive(Default)]
pub struct HarnessOs(HashMap<ProgramId, WapcHostAsync>);

impl HarnessOs {
    /// This is responsible for instantiating the host process needed to load the program
    pub async fn new(program_id: ProgramId, program: &[u8]) -> Result<Self> {
        let engine = wasmtime_provider::WasmtimeEngineProviderBuilder::new()
            .module_bytes(program)
            .build_async()?;

        Ok(Self(HashMap::from([(
            program_id,
            WapcHostAsync::new(Box::new(engine), None).await?,
        )])))
    }

    /// Returns the list of program identifiers that are currently loaded in the device.
    pub fn program_ids(&self) -> Vec<ProgramId> {
        self.0.keys().cloned().collect()
    }

    /// This calls the operation and returns the result or appropriate errors to the caller.
    /// Note that serde to/from bytes is done inherently in the compiled program which uses candid
    pub async fn call_operation(
        &self,
        program_id: &ProgramId,
        operation: &str,
        payload: &[u8],
    ) -> Result<Vec<u8>> {
        match self.0.get(program_id) {
            Some(program) => Ok(program.call(operation, payload).await?),
            None => Err(Error::Internal {
                message: "the program could not be found".to_string(),
                inner: None,
            }),
        }
    }

    /// Adds a new program to the device.
    pub async fn add_program(&mut self, program_id: ProgramId, program: &[u8]) -> Result<()> {
        let engine = wasmtime_provider::WasmtimeEngineProviderBuilder::new()
            .module_bytes(program)
            .build_async()?;

        _ = self.0.insert(
            program_id,
            WapcHostAsync::new(Box::new(engine), None).await?,
        );
        Ok(())
    }

    /// Removes a program from the set, noop if not found.
    pub fn remove_program(&mut self, program_id: &ProgramId) {
        let _ = self.0.remove(program_id);
    }
}
