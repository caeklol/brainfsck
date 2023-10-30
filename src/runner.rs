use std::io::Write;

use crate::parser::{parse, Instruction, OpCode, ParseError};

pub struct Environment {
    tape: Vec<std::num::Wrapping<u8>>,
    ptr: usize,
}

impl Environment {
    pub fn new(tape_size: usize) -> Self {
        Environment {
            tape: vec![std::num::Wrapping(0); tape_size],
            ptr: 0,
        }
    }

    fn execute(&mut self, instructions: &[Instruction]) {
        for instruction in instructions {
            match instruction.opcode {
                OpCode::Left => {
                    self.ptr -= instruction.amount;
                }
                OpCode::Right => {
                    self.ptr += instruction.amount;
                }
                OpCode::Inc => self.tape[self.ptr] += instruction.amount as u8,
                OpCode::Dec => self.tape[self.ptr] -= instruction.amount as u8,
                OpCode::Loop => {
                    while self.tape[self.ptr] > std::num::Wrapping(0) {
                        self.execute(&instruction.instructions)
                    }
                }
                OpCode::Query => {
                    // i forgot rust for loop syntax
                    let mut i = 0;
                    while i < instruction.amount {
                        let char = console::Term::stdout()
                            .read_char()
                            .expect("Terminal is not user attended");
                        self.tape[self.ptr] = std::num::Wrapping(char as u8);
                        i += 1;
                    }
                }
                OpCode::Print => {
                    let char = (self.tape[self.ptr].0 as char).to_string();
                    print!("{}", char.repeat(instruction.amount));
                    std::io::stdout().flush().unwrap();
                }
            }
        }
    }

    pub fn evaluate(&mut self, input: &str) -> Result<(), ParseError> {
        let instructions = parse(input)?;
        self.execute(&instructions);

        return Ok(());
    }
}
