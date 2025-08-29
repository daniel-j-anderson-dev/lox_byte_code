/// A single instruction to the VM
#[derive(Debug, Clone, Copy)]
pub enum Opcode {
    /// Return from the current function
    Return,
}
impl Opcode {
    pub const fn new(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(Self::Return),
            _ => None,
        }
    }
    pub const fn as_byte(&self) -> u8 {
        *self as _
    }
}
