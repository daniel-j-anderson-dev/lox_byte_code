use crate::{Opcode, chunks, value::Value};
use my_vec::DynamicSizeArray;

pub struct Chunk {
    source: DynamicSizeArray<u8>,
    constants: DynamicSizeArray<Value>,
}
// constructors
impl Chunk {
    pub const fn new() -> Self {
        Self {
            source: DynamicSizeArray::new(),
            constants: DynamicSizeArray::new(),
        }
    }
}

// accessors
impl Chunk {
    pub const fn as_bytes(&self) -> &[u8] {
        self.source.as_slice()
    }
    pub const fn length(&self) -> usize {
        self.source.length()
    }
}

// helpers
impl Chunk {
    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);

        let mut offset = 0;
        while offset < self.source.length() {
            offset = match self.disassemble_instruction(offset) {
                Ok(offset) => offset,
                Err(error) => match error {
                    DisassembleInstructionError::InvalidOpcode(byte) => {
                        println!("Unrecognized opcode: {}", byte);
                        continue;
                    }
                    DisassembleInstructionError::IndexOutOfBounds { length, index } => {
                        println!("index {} is out of bounds!. length: {}", index, length);
                        continue;
                    }
                },
            };
        }
    }

    pub fn disassemble_instruction(
        &self,
        offset: usize,
    ) -> Result<usize, DisassembleInstructionError> {
        use DisassembleInstructionError::*;
        print!("{:04} ", offset);

        let byte = self.source.get(offset).copied().ok_or(IndexOutOfBounds {
            length: self.source.length(),
            index: offset,
        })?;
        let opcode = Opcode::new(byte).ok_or(InvalidOpcode(byte))?;

        Ok(match opcode {
            Opcode::Return => self.simple_instruction("OP_RETURN", offset),
        })
    }

    pub fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{}", name);
        offset + 1
    }
}

// mutators
impl Chunk {
    pub const fn as_bytes_mutable(&mut self) -> &mut [u8] {
        self.source.as_mutable_slice()
    }
    pub fn push(&mut self, instruction: Opcode) {
        self.push_byte(instruction.as_byte());
    }
    fn push_byte(&mut self, byte: u8) {
        self.source.push(byte);
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum DisassembleInstructionError {
    InvalidOpcode(u8),
    IndexOutOfBounds { length: usize, index: usize },
}
impl core::fmt::Display for DisassembleInstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl core::error::Error for DisassembleInstructionError {}
