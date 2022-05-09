use crate::runtime::value::Type;
use crate::CompiledProgram;

use super::chunk::{Chunk, Instr};
use super::value::Value;

#[derive(Debug, Clone)]
pub enum RuntimeError {
    DivisionByZero,
    TypeError { expected: Type, found: Type },
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
                read!().instr()
            };
        }

        macro_rules! read_u8 {
            () => {
                read!().byte()
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

        macro_rules! peek {
            () => {{
                self.value_stack.last().unwrap()
            }};
        }

        macro_rules! pop {
            () => {{
                self.value_stack.pop().unwrap()
            }};
        }

        macro_rules! push {
            ($item:expr) => {{
                self.value_stack.push($item);
            }};
        }

        macro_rules! integer_op {
            ($op:tt) => {
                op!(integer $op -> Integer)
            };
        }

        macro_rules! bool_op {
            ($op:tt) => {
                op!(bool $op -> Bool)
            };
        }

        macro_rules! op {
            ($in_type:tt $op:tt -> $out_type:tt) => {{
                let b = pop!().$in_type()?;
                let a = pop!().$in_type()?;
                push!(Value::$out_type(a $op b));
            }};
            ($a_type:tt $op:tt $b_type:tt -> $out_type:tt) => {{
                let b = pop!().$b_type()?;
                let a = pop!().$a_type()?;
                push!(Value::$out_type(a $op b));
            }};
        }

        macro_rules! vm_assert {
            ($case:expr, $err:ident) => {{
                if !$case {
                    return Err(RuntimeError::$err);
                }
            }};
        }

        loop {
            let instr = read_instr!();
            match instr {
                Instr::OrBool => bool_op!(||),
                Instr::AndBool => bool_op!(&&),
                Instr::NotBool => {
                    let a = !pop!().bool()?;
                    push!(Value::Bool(a))
                }

                Instr::Equal => op!(integer == -> Bool),
                Instr::LT => op!(integer < -> Bool),
                Instr::LTEqual => op!(integer <= -> Bool),

                Instr::AddInt => integer_op!(+),
                Instr::SubInt => integer_op!(-),
                Instr::MulInt => integer_op!(*),
                Instr::DivInt => {
                    vm_assert!(peek!().integer()? != 0, DivisionByZero);
                    integer_op!(/);
                }
                Instr::ModInt => {
                    vm_assert!(peek!().integer()? != 0, DivisionByZero);
                    integer_op!(%);
                }
                Instr::PowInt => {
                    let b = pop!().integer()? as u32;
                    let a = pop!().integer()?;
                    push!(Value::Integer(a.pow(b)));
                }

                Instr::LoadConstantU8 => {
                    let constant = frame.chunk.constants[read_u8!() as usize].clone();
                    push!(constant);
                }
                Instr::LoadConstantU32 => {
                    let constant = frame.chunk.constants[read_u32!() as usize].clone();
                    push!(constant);
                }

                Instr::LoadUnit => {
                    push!(Value::Unit)
                }

                Instr::LoadLocal => {
                    push!(self.value_stack[read_u8!() as usize].clone())
                }
                Instr::StoreLocal => {
                    self.value_stack[read_u8!() as usize] = peek!().clone();
                }

                Instr::Pop => drop(pop!()),

                Instr::JumpU32 => {
                    frame.ip = read_u32!();
                }
                Instr::JumpNotU32 => {
                    let dest = read_u32!();
                    if !pop!().bool()? {
                        frame.ip = dest;
                    }
                }

                Instr::Return => break,

                Instr::DebugPrint => println!("{}", pop!()),
            }
        }

        Ok(())
    }
}
