use core::fmt::Display;

use my_vec::DynamicSizeArray;

pub struct Chunk {
    source: DynamicSizeArray<u8>,
}
impl Chunk {
    pub const fn new() -> Self {
        Self {
            source: DynamicSizeArray::new(),
        }
    }

    pub fn disassemble(&self, offset: usize) {
        todo!()
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}
