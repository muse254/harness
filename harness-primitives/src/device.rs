use std::collections::HashMap;

use crate::harness_os::ProgramId;

/// Device holds the presets that are important for the `Arbiter` smart contract scheduler and also for
/// book-keeping purposes.
pub struct Device {
    /// The maximum program size in bytes that the device can accept to load. todo/fixme? research
    program_limit: u64,
    /// The count in cycles the device has processed. todo/fixme? research
    cycles_counter: u128,
    /// A set of programs and their respective procedures that are loaded in the device.
    programs: HashMap<ProgramId, Vec<String>>,
}

impl Device {
    pub fn new(program_limit: u64) -> Self {
        return Device {
            program_limit,
            cycles_counter: 0,
            programs: HashMap::new(),
        };
    }

    /// Adds a program to the device.
    pub fn add_program(&mut self, program_id: ProgramId, procedures: &[String]) {
        self.programs.insert(program_id, procedures.to_vec());
    }

    /// Removes a program from the set, noop if not found.
    pub fn remove_program(&mut self, program_id: &ProgramId) {
        self.programs.remove(program_id);
    }
}
