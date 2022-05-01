use std::fmt::Debug;

use super::value::Value;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Instr {
    OrBool,
    AndBool,
    NotBool,

    Equal,
    LT,
    LTEqual,

    AddInt,
    SubInt,
    MulInt,
    DivInt,
    Mod,
    Pow,

    LoadConstantU8,
    LoadConstantU32,

    Pop,

    Return,

    DebugPrint,
}

#[derive(Clone, Copy)]
pub union Opcode {
    pub instr: Instr,
    pub byte: u8,
}

#[derive(Default)]
pub struct Chunk {
    pub code: Vec<Opcode>,
    pub constants: Vec<Value>,
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Chunk")
    }
}
