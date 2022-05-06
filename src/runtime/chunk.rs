use strum::EnumCount;

use std::fmt::{Debug, Formatter};

use super::value::Value;

#[repr(u8)]
#[derive(Debug, Clone, Copy, strum_macros::EnumCount)]
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
    LoadUnit,

    Pop,

    JumpU32,
    JumpNotU32,

    Return,

    DebugPrint,
}

#[derive(Clone, Copy)]
pub union Opcode {
    instr: Instr,
    byte: u8,
}

impl From<Instr> for Opcode {
    fn from(instr: Instr) -> Self {
        Self { instr }
    }
}

impl From<u8> for Opcode {
    fn from(byte: u8) -> Self {
        Self { byte }
    }
}

impl Opcode {
    pub fn set_instr(&mut self, instr: Instr) {
        self.instr = instr;
    }

    pub fn set_byte(&mut self, byte: u8) {
        self.byte = byte;
    }

    #[inline(always)]
    pub fn instr(&self) -> Instr {
        debug_assert!(self.could_be_instr());
        unsafe { self.instr }
    }

    pub fn instr_safe(&self) -> Option<Instr> {
        if self.could_be_instr() {
            Some(unsafe { self.instr })
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn byte(&self) -> u8 {
        unsafe { self.byte }
    }

    fn could_be_instr(&self) -> bool {
        self.byte() <= Instr::COUNT as u8
    }
}

impl Debug for Opcode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let instr = match self.instr_safe() {
            Some(instr) => format!(" : {:?}", instr),
            None => String::new(),
        };
        write!(f, "Opcode({}{})", self.byte(), instr)
    }
}

#[derive(Default, Debug)]
pub struct Chunk {
    pub code: Vec<Opcode>,
    pub constants: Vec<Value>,
}
