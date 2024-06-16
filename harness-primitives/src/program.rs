/// This struct represents a program that can be loaded into the device.
pub struct Program {
    pub id: crate::ProgramId,
    pub wasm: &'static [u8],
    pub schema: crate::internals::Schema,
}
