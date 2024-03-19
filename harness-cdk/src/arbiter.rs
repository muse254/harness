struct Arbiter {
    harness_code: Vec<u8>,
}

/*
1. After building the harness code, have a way to list all functions that were built.
2. 
*/

impl Arbiter {
    pub(crate) fn new() -> Result<Self> {
        let code_path = env!("HARNESS_WASM_PATH");

        Self { harness_code }
    }

    pub fn harness_code(&self) -> Vec<u8> {
        todo!()
    }
}


