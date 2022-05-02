use crate::CompiledProgram;

use super::chunk::{Chunk, Instr};
use super::value::Value;

#[derive(Debug, Clone)]
pub enum RuntimeError {
    DivisionByZero,
}

pub struct CallFrame {
    chunk: Chunk,
    ip: usize,
}

impl CallFrame {
    pub fn new(chunk: Chunk) -> Self {
        Self { chunk, ip: 0 }
    }
}

#[derive(Default)]
pub struct VM {
    call_stack: Vec<CallFrame>,
    value_stack: Vec<Value>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            call_stack: vec![],
            value_stack: vec![],
        }
    }

    pub fn run(&mut self, module: CompiledProgram) -> Result<(), RuntimeError> {
        let mut frame = CallFrame::new(module.chunk);

        macro_rules! read {
            () => {{
                let opcode = frame.chunk.code[frame.ip];
                frame.ip += 1;
                opcode
            }};
        }

        macro_rules! read_instr {
            () => {
                unsafe { read!().instr }
            };
        }

        macro_rules! read_u8 {
            () => {
                unsafe { read!().byte }
            };
        }

        macro_rules! read_u32 {
            () => {{
                let mut n = read_u8!() as usize;
                n = (n << 8) + read_u8!() as usize;
                n = (n << 8) + read_u8!() as usize;
                n = (n << 8) + read_u8!() as usize;
                n
            }};
        }

        macro_rules! integer_op {
            ($op:tt) => {{
                let b = self.value_stack.pop().unwrap().integer();
                let a = self.value_stack.pop().unwrap().integer();

                self.value_stack.push(Value::Integer(a $op b));
            }};
        }

        loop {
            let instr = read_instr!();
            match instr {
                Instr::AddInt => integer_op!(+),
                Instr::SubInt => integer_op!(-),
                Instr::MulInt => integer_op!(*),
                Instr::DivInt => {
                    if self.value_stack.last().unwrap().integer() == 0 {
                        return Err(RuntimeError::DivisionByZero);
                    }
                    integer_op!(/)
                }

                Instr::LoadConstantU8 => {
                    let constant = frame.chunk.constants[read_u8!() as usize].clone();
                    self.value_stack.push(constant);
                }

                Instr::LoadConstantU32 => {
                    let constant = frame.chunk.constants[read_u32!() as usize].clone();
                    self.value_stack.push(constant);
                }

                Instr::Pop => drop(self.value_stack.pop()),

                Instr::Return => break,

                Instr::DebugPrint => println!("{:?}", self.value_stack.pop().unwrap()),

                _ => todo!("implement rest of instructions"),
            }
        }

        Ok(())
    }
}
