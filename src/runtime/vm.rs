use crate::runtime::value::Type;
use crate::CompiledProgram;

use super::chunk::{Chunk, Instr};
use super::value::Value;

#[derive(Debug, Clone)]
pub enum RuntimeError {
    IntegerOverflow,
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
            ($ty:ident) => {{
                match self.value_stack.pop().unwrap() {
                    Value::$ty(n) => Ok(n),
                    other => Err(RuntimeError::TypeError {
                        expected: Type::$ty,
                        found: other.ty(),
                    }),
                }
            }};
        }

        macro_rules! binary_op {
            ($op:tt, $ty_in:ident -> $ty_out:ident) => {{
                let b = pop!($ty_in)?;
                let a = pop!($ty_in)?;
                push!(Value::$ty_out(a $op b));
            }};
        }

        macro_rules! integer_op {
            ($op:ident, $err:ident) => {{
                let b = pop!(Integer)?;
                let a = pop!(Integer)?;

                let c = a.$op(b).ok_or(RuntimeError::$err)?;
                push!(Value::Integer(c));
            }};
        }

        macro_rules! push {
            ($item:expr) => {{
                self.value_stack.push($item);
            }};
        }

        loop {
            let instr = read_instr!();
            match instr {
                Instr::OrBool => binary_op!(||, Bool -> Bool),
                Instr::AndBool => binary_op!(&&, Bool -> Bool),
                Instr::NotBool => {
                    let a = pop!(Bool)?;
                    push!(Value::Bool(!a))
                }

                Instr::Equal => binary_op!(==, Integer -> Bool),
                Instr::LT => binary_op!(<, Integer -> Bool),
                Instr::LTEqual => binary_op!(<=, Integer -> Bool),

                Instr::AddInt => integer_op!(checked_add, IntegerOverflow),
                Instr::SubInt => integer_op!(checked_sub, IntegerOverflow),
                Instr::MulInt => integer_op!(checked_mul, IntegerOverflow),
                Instr::DivInt => integer_op!(checked_div, IntegerOverflow),
                Instr::ModInt => integer_op!(checked_rem, IntegerOverflow),
                Instr::PowInt => {
                    let b = pop!(Integer)? as u32;
                    let a = pop!(Integer)?;
                    push!(Value::Integer(a.pow(b)));
                }
                Instr::NegInt => {
                    let a = pop!(Integer)?;
                    push!(Value::Integer(-a))
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
                    if !pop!(Bool)? {
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
