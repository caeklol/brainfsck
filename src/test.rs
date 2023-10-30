use crate::parser;

#[cfg(test)]
fn reconstruct(instructions: Vec<parser::Instruction>) -> String {
    let mut output = String::new();
    for instruction in instructions {
        output.push_str(&match instruction.opcode {
            parser::OpCode::Left => "<".repeat(instruction.amount),
            parser::OpCode::Right => ">".repeat(instruction.amount),
            parser::OpCode::Inc => "+".repeat(instruction.amount),
            parser::OpCode::Dec => "-".repeat(instruction.amount),
            parser::OpCode::Print => ".".repeat(instruction.amount),
            parser::OpCode::Query => ",".repeat(instruction.amount),
            parser::OpCode::Loop => format!("[{}]", reconstruct(instruction.instructions)),
        });
    }
    return output;
}

#[test]
fn test_hello_world_parse() {
    let hello_world = ">++++++++[-<+++++++++>]<.>>+>-[+]++>++>+++[>[->+++<<+++>]<<]>-----.>->+++..+++.>-.<<+[>[+>+]>>]<--------------.>>.+++.------.--------.>+.>+.";
    let instructions = parser::parse(hello_world).unwrap();
    let reconstruction = &reconstruct(instructions);

    assert_eq!(reconstruction, hello_world);
}

#[test]
fn test_nested_loop() {
    let hello_world = ">[>[+]<]<";
    let instructions = parser::parse(hello_world).unwrap();
    let reconstruction = &reconstruct(instructions);

    assert_eq!(reconstruction, hello_world);
}

#[test]
fn test_empty_nested_loop() {
    let hello_world = "[[]]";
    let instructions = parser::parse(hello_world).unwrap();
    let reconstruction = &reconstruct(instructions);

    assert_eq!(reconstruction, hello_world);
}

#[test]
fn test_one_element_loop() {
    let hello_world = "[+]";
    let instructions = parser::parse(hello_world).unwrap();
    let reconstruction = &reconstruct(instructions);

    assert_eq!(reconstruction, hello_world);
}

#[test]
fn test_zeroarithmetic_loop() {
    let hello_world = "+[>[>]>[<]<]";
    let instructions = parser::parse(hello_world).unwrap();
    let reconstruction = &reconstruct(instructions);

    assert_eq!(reconstruction, hello_world);
}