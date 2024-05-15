use harness_primitives::program::{Program, ProgramId};

/// Device holds the presets that are important for the `Arbiter` smart contract scheduler and also for
/// book-keeping purposes.
pub struct Device {
    /// The maximum program size in bytes that the device can accept to load
    program_limit: u64,
    /// The count in cycles the device has processed. fixme? research
    cycles_counter: u128,
    /// A set of program loaded on the device.
    programs: Vec<Program>,
}

impl Device {
    pub fn new(program_limit: u64) -> Self {
        return Device {
            program_limit,
            cycles_counter: 0,
            programs: vec![],
        };
    }

    pub fn add_program(&mut self, program: Program) {
        self.programs.push(program)
    }

    /// Removes a program from the set, noop if not found.
    pub fn remove_program(&mut self, program_id: ProgramId) {
        if let Some(pos) = self
            .programs
            .iter()
            .position(|v| v.program_id().eq(&program_id))
        {
            self.programs.remove(pos);
        }
    }
}
