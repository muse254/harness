use wapc::WapcHost;

use crate::error::Result;

/// A program that has been loaded.
pub struct Program {
    /// The program identifier as used in the harness network.
    program_id: ProgramId,
    // A WebAssembly host runtime for waPC-compliant modules.
    host_instance: WapcHost,
}

impl Program {
    /// This is responsible for instantiating the host process needed to load the program
    pub fn new(id: &str, program: &[u8]) -> Result<Self> {
        let engine = wasmtime_provider::WasmtimeEngineProviderBuilder::new()
            .module_bytes(&program)
            .build()?;

        Ok(Program {
            program_id: id.parse()?,
            host_instance: WapcHost::new(
                Box::new(engine),
                Some(Box::new(move |_a, _b, _c, _d, _e| Ok(vec![]))), // todo?
            )?,
        })
    }

    pub fn program_id(&self) -> &ProgramId {
        &self.program_id
    }

    /// This calls the method and returns the result or appropriate errors to the caller.
    /// Note that serde to/from bytes is done inherently in the compiled program which uses candid
    pub fn call_method(&self, method: &str, payload: &[u8]) -> Result<Vec<u8>> {
        Ok(self.host_instance.call(method, payload)?)
    }
}

mod program_id {
    use std::str::FromStr;

    use crate::error::Error;

    /// The program identifier. It should be a human-readable identifier on the Harness network.
    /// TODO: parse? `<network>.<account_id>.<program_name>`
    #[derive(Eq, Ord, Hash, Clone, Debug, PartialEq, PartialOrd)]
    pub struct ProgramId(Box<str>);

    impl TryFrom<String> for ProgramId {
        type Error = Error;

        fn try_from(program_id: String) -> std::result::Result<Self, Self::Error> {
            Ok(Self(program_id.into_boxed_str()))
        }
    }

    impl FromStr for ProgramId {
        type Err = Error;

        fn from_str(program_id: &str) -> Result<Self, Self::Err> {
            Ok(Self(program_id.into()))
        }
    }
}

pub use program_id::ProgramId;
