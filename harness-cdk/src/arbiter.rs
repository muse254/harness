//! This is is where the harness program is loaded at compile time, we create the arbiter to arbiter operations of the harness program.
use std::cell::{Cell, RefCell};

use harness_macros::{get_binary__, get_schema};
use harness_primitives::{
    error::{Error, Result},
    internals::Schema,
    program::{Program, ProgramId},
};

struct Arbiter {
    // The collection of device urls that have been registered with the arbiter.
    devices: Vec<String>,
    // The harness program that is loaded into the arbiter at compile time.
    program: Program,
}

thread_local! {
    static NEXT_DEVICE_ID: Cell<usize> = const{ Cell::new(0)};// rudimentary round robin scheduling

    #[allow(clippy::large_stack_frames)]
    static ARBITER: RefCell<Arbiter> = RefCell::new( Arbiter {
        devices: Vec::new(),
        program: Program { schema: get_schema!(), wasm: get_binary__!() },
    });
}

/// This is redirection that does not expose the ARBITER to the user.
pub struct StateAccessor;

impl StateAccessor {
    pub fn add_device(url: String) {
        ARBITER.with(|arbiter| arbiter.borrow_mut().devices.push(url));
    }

    pub fn get_program_code() -> Vec<u8> {
        ARBITER.with(|arbiter| arbiter.borrow().program.wasm.to_vec())
    }

    pub fn get_program_id() -> ProgramId {
        ARBITER.with(|arbiter| ProgramId::new(arbiter.borrow().program.schema.program.clone()))
    }

    pub fn get_next_device() -> Result<String> {
        ARBITER.with(|arbiter| {
            let devices = &arbiter.borrow().devices;
            if devices.is_empty() {
                return Err(Error::Internal {
                    message: "No devices registered".to_string(),
                    inner: None,
                });
            }

            let idx = NEXT_DEVICE_ID.with(|next_device_id| {
                let id = next_device_id.get();
                next_device_id.set((id + 1) % devices.len());
                id
            });

            Ok(devices[idx].clone())
        })
    }

    pub fn get_devices() -> Vec<String> {
        ARBITER.with(|arbiter| arbiter.borrow().devices.clone())
    }

    pub fn remove_device(url: String) {
        ARBITER.with(|arbiter| {
            let devices = &mut arbiter.borrow_mut().devices;
            if let Some(idx) = devices.iter().position(|x| x == &url) {
                devices.remove(idx);
            }
        });
    }

    pub fn get_schema() -> Schema {
        ARBITER.with(|arbiter| arbiter.borrow().program.schema.clone())
    }
}
