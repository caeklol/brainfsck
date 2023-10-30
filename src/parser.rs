extern crate custom_error;
use custom_error::custom_error;

custom_error! {pub ParseError
    MismatchedBeginLoop = "Mismatched '['!",
    MismatchedEndLoop = "Mismatched ']'!"
}

#[derive(Debug, Clone, Copy)]
enum Token {
    Left,
    Right,
    Inc,
    Dec,
    Print,
    Query,
    BeginLoop,
    EndLoop,
}

impl Token {
    pub fn tokenize(input: char) -> Option<Token> {
        match input {
            '<' => Some(Token::Left),
            '>' => Some(Token::Right),
            '+' => Some(Token::Inc),
            '-' => Some(Token::Dec),
            '.' => Some(Token::Print),
            ',' => Some(Token::Query),
            '[' => Some(Token::BeginLoop),
            ']' => Some(Token::EndLoop),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OpCode {
    Left,
    Right,
    Inc,
    Dec,
    Print,
    Query,
    Loop,
}

#[derive(Debug)]
pub struct Instruction {
    pub opcode: OpCode,
    pub amount: usize,
    pub instructions: Vec<Instruction>,
}

fn tokenize(input: &str) -> Vec<Token> {
    return input.chars().filter_map(Token::tokenize).collect();
}

fn to_instructions(tokens: Vec<Token>) -> Result<Vec<Instruction>, ParseError> {
    let mut instructions = Vec::new();
    let mut index = 0;

    let mut last_opcode: Option<OpCode> = None;
    let mut opcode_amount: usize = 1;

    while index < tokens.len() {
        let token = tokens.get(index).unwrap();

        match token {
            Token::BeginLoop => {
                if last_opcode.is_some() {
                    instructions.push(Instruction {
                        opcode: last_opcode.unwrap(),
                        amount: opcode_amount,
                        instructions: Vec::new()
                    });

                    last_opcode = None;
                    opcode_amount = 1;
                }
            }

            _ => {}
        }

        match token {
            Token::Left | Token::Right | Token::Inc | Token::Dec | Token::Print | Token::Query => {
                let opcode = match token {
                    Token::Left => OpCode::Left,
                    Token::Right => OpCode::Right,
                    Token::Inc => OpCode::Inc,
                    Token::Dec => OpCode::Dec,
                    Token::Print => OpCode::Print,
                    Token::Query => OpCode::Query,
                    Token::BeginLoop => panic!("What?"),
                    Token::EndLoop => panic!("What?"),
                };

                if last_opcode.is_none() {
                    last_opcode = Some(opcode);
                    opcode_amount = 1;
                } else {
                    if last_opcode.unwrap() == opcode {
                        opcode_amount += 1;
                    } else {
                        instructions.push(Instruction {
                            opcode: last_opcode.unwrap(),
                            amount: opcode_amount,
                            instructions: Vec::new()
                        });

                        last_opcode = Some(opcode);
                        opcode_amount = 1;
                    }
                }
            }

            Token::BeginLoop => {
                let mut search_depth = 1;
                let mut search_index = index + 1;
                let mut loop_tokens = Vec::new();

                while search_depth > 0 {
                    if search_index > tokens.len() {
                        return Err(ParseError::MismatchedBeginLoop);
                    }

                    match tokens[search_index] {
                        Token::BeginLoop => search_depth += 1,
                        Token::EndLoop => search_depth -= 1,
                        _ => (),
                    }

                    if search_depth == 0 {
                        break;
                    }

                    loop_tokens.push(tokens[search_index]);
                    search_index += 1;
                    index += 1;
                }

                instructions.push(Instruction {
                    instructions: to_instructions(loop_tokens)?,
                    opcode: OpCode::Loop,
                    amount: 1
                });
                index += 1;
            }
            Token::EndLoop => return Err(ParseError::MismatchedEndLoop),
        }

        index += 1;
    }

    if last_opcode.is_some() {
        instructions.push(Instruction {
            opcode: last_opcode.unwrap(),
            amount: opcode_amount,
            instructions: Vec::new()
        });
    }

    return Ok(instructions);
}

pub fn parse(input: &str) -> Result<Vec<Instruction>, ParseError> {
    let tokens = tokenize(input);
    let instructions: Vec<Instruction> = to_instructions(tokens)?;

    return Ok(instructions);
}
