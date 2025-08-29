use crate::{Opcode, chunks::Chunk};

#[test]
fn test_chunk() {
    let mut chunk = Chunk::new();
    chunk.push(Opcode::Return);
    chunk.disassemble("test chunk");
    
}