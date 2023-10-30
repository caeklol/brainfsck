use custom_error::custom_error;
use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    memory_buffer::MemoryBuffer,
    module::{Linkage, Module},
    targets::{
        CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine, TargetTriple,
    },
    values::{FunctionValue, PointerValue},
    AddressSpace, OptimizationLevel,
};

use crate::parser::{parse, Instruction, OpCode, ParseError};

custom_error! {pub LLVMError
    TargetMachine = "Failed to identify get machine!",
    ObjectCreation { llvm_err: String } = "Failed to create object file!",
    ModuleCreation { llvm_err: String } = "Failed creating module!",
}

#[derive(Debug)]
pub enum CompilerError {
    Compile(LLVMError),
    Parse(ParseError),
}

impl From<ParseError> for CompilerError {
    fn from(e: ParseError) -> Self {
        CompilerError::Parse(e)
    }
}

impl From<LLVMError> for CompilerError {
    fn from(e: LLVMError) -> Self {
        CompilerError::Compile(e)
    }
}

fn get_target_machine() -> Option<TargetMachine> {
    Target::initialize_x86(&InitializationConfig::default());

    let opt = OptimizationLevel::Aggressive;
    let reloc = RelocMode::Default;
    let model = CodeModel::Default;
    let target = Target::from_name("x86-64").unwrap();
    return target.create_target_machine(
        &TargetTriple::create("x86_64-pc-linux-gnu"),
        "x86-64",
        "+avx2",
        opt,
        reloc,
        model,
    );
}

fn build_instructions(
    instructions: &Vec<Instruction>,
    context: &Context,
    builder: &Builder,
    tape_size: u32,
    index_ptr: PointerValue,
    tape_ptr: PointerValue,
    putchar: FunctionValue,
    getchar: FunctionValue,
    main: FunctionValue,
    curr_block: BasicBlock,
) {
    let i32_type = context.i32_type();
    let i8_type = context.i8_type();

    let tape_type = i8_type.array_type(tape_size.try_into().unwrap());

    for instruction in instructions {
        match instruction.opcode {
            OpCode::Loop => {
                let loop_check = context.insert_basic_block_after(curr_block, "loop_check");
                let loop_contents = context.insert_basic_block_after(loop_check, "loop_contents");
                let loop_exit = context.insert_basic_block_after(loop_contents, "loop_exit");

                builder.build_unconditional_branch(loop_check);

                builder.position_at_end(loop_check);

                let current_index = builder
                    .build_load(i32_type, index_ptr, "current_index")
                    .into_int_value();
                let cell_data_ptr = unsafe {
                    builder.build_in_bounds_gep(
                        tape_type.ptr_type(AddressSpace::default()),
                        tape_ptr,
                        &[current_index],
                        "cell_data_ptr",
                    )
                };

                let cell_data = builder
                    .build_load(i8_type, cell_data_ptr, "cell_data")
                    .into_int_value();
                let cell_data_ext =
                    builder.build_int_s_extend(cell_data, i32_type, "cell_data_ext");
                let conditional = builder.build_int_compare(
                    inkwell::IntPredicate::NE,
                    cell_data_ext,
                    i32_type.const_zero(),
                    "loop_conditional",
                );

                builder.build_conditional_branch(conditional, loop_contents, loop_exit);

                builder.position_at_end(loop_contents);
                build_instructions(
                    &instruction.instructions,
                    context,
                    builder,
                    tape_size,
                    index_ptr,
                    tape_ptr,
                    putchar,
                    getchar,
                    main,
                    loop_contents,
                );
                builder.build_unconditional_branch(loop_check);

                builder.position_at_end(loop_exit);
            }
            OpCode::Left | OpCode::Right => {
                let offset = context
                    .i32_type()
                    .const_int(instruction.amount.try_into().unwrap(), false);
                let current_index = builder
                    .build_load(i32_type, index_ptr, "current_index")
                    .into_int_value();
                let new_index = match instruction.opcode {
                    OpCode::Right => {
                        builder.build_int_nsw_add(current_index, offset, "replacement_index")
                    }
                    OpCode::Left => {
                        builder.build_int_nsw_sub(current_index, offset, "replacement_index")
                    }
                    _ => unimplemented!(),
                };
                builder.build_store(index_ptr, new_index);
            }

            OpCode::Inc | OpCode::Dec => {
                let amount = context
                    .i32_type()
                    .const_int((instruction.amount as u32).into(), false);
                let current_index = builder
                    .build_load(i32_type, index_ptr, "current_index")
                    .into_int_value();
                let cell_data_ptr = unsafe {
                    builder.build_in_bounds_gep(
                        tape_type.ptr_type(AddressSpace::default()),
                        tape_ptr,
                        &[current_index],
                        "cell_data_ptr",
                    )
                };

                let cell_data = builder
                    .build_load(i8_type, cell_data_ptr, "cell_data")
                    .into_int_value();
                let cell_data_ext =
                    builder.build_int_s_extend(cell_data, i32_type, "cell_data_ext");
                let new_cell_data = match instruction.opcode {
                    OpCode::Dec => {
                        builder.build_int_nsw_sub(cell_data_ext, amount, "new_cell_data")
                    }
                    OpCode::Inc => {
                        builder.build_int_nsw_add(cell_data_ext, amount, "new_cell_data")
                    }
                    _ => unimplemented!(),
                };
                let t_new_cell_data =
                    builder.build_int_truncate(new_cell_data, i8_type, "cell_data_trunc");

                builder.build_store(cell_data_ptr, t_new_cell_data);
            }
            OpCode::Print => {
                for _ in 0..instruction.amount {
                    let current_index = builder
                        .build_load(i32_type, index_ptr, "current_index")
                        .into_int_value();
                    let cell_data_ptr = unsafe {
                        builder.build_in_bounds_gep(
                            tape_type.ptr_type(AddressSpace::default()),
                            tape_ptr,
                            &[current_index],
                            "cell_data_ptr",
                        )
                    };

                    let cell_data = builder
                        .build_load(i8_type, cell_data_ptr, "cell_data")
                        .into_int_value();
                    let i32_cell_data =
                        builder.build_int_z_extend(cell_data, i32_type, "cell_data_zext");

                    builder.build_call(putchar, &[i32_cell_data.into()], "underscore");
                }
            }
            OpCode::Query => {}
        }
    }
}

fn gen_module(
    context: &Context,
    instructions: Vec<Instruction>,
    tape_size: u32,
) -> Result<Module, LLVMError> {
    let module = context.create_module("brainfsck");

    let builder = context.create_builder();

    let i8_type = context.i8_type();
    let i32_type = context.i32_type();

    // puts
    let putchar_type = i32_type.fn_type(&[i32_type.into()], false);
    let putchar = module.add_function("putchar", putchar_type, Some(Linkage::External));

    // getchar
    let getchar_type = i32_type.fn_type(&[], false);
    let getchar = module.add_function("getchar", getchar_type, Some(Linkage::External));

    // main
    let main_type = i8_type.fn_type(&[], false);
    let main = module.add_function("main", main_type, None);

    // beginning block
    let block = context.append_basic_block(main, "entry");
    builder.position_at_end(block);

    // tape
    let tape_size_llvm = i32_type.const_int(tape_size.into(), false);
    let tape_type = i8_type.array_type(tape_size);
    let tape_ptr = builder.build_alloca(tape_type, "tape");
    builder
        .build_memset(tape_ptr, 1, i8_type.const_zero(), tape_size_llvm)
        .expect("Memset build failed");

    // zero index
    let index_ptr = builder.build_alloca(i32_type, "index");
    builder.build_store(index_ptr, i32_type.const_zero());

    build_instructions(
        &instructions,
        context,
        &builder,
        tape_size,
        index_ptr,
        tape_ptr,
        putchar,
        getchar,
        main,
        block,
    );

    let current_index = builder
        .build_load(i8_type, index_ptr, "current_index")
        .into_int_value();
    let cell_data_ptr = unsafe {
        builder.build_in_bounds_gep(
            tape_type.ptr_type(AddressSpace::default()),
            tape_ptr,
            &[current_index],
            "cell_data_ptr",
        )
    };

    let cell_data = builder
        .build_load(i8_type, cell_data_ptr, "cell_data")
        .into_int_value();
    builder.build_return(Some(&cell_data));

    return Ok(module);
}

pub fn gen_object(input: &str, tape_size: usize) -> Result<MemoryBuffer, CompilerError> {
    // stolen from:
    // https://schroer.ca/2021/10/30/cw-llvm-backend/

    let instructions = parse(input)?;

    let context = Context::create();
    let module = gen_module(&context, instructions, tape_size.try_into().unwrap())?;
    let target = get_target_machine();

    // println!("{}", module.to_string());

    if target.is_none() {
        return Err(CompilerError::Compile(LLVMError::TargetMachine));
    }

    let object_bytes = match target
        .unwrap()
        .write_to_memory_buffer(&module, FileType::Object)
    {
        Ok(buf) => buf,
        Err(err) => {
            return Err(CompilerError::Compile(LLVMError::ObjectCreation {
                llvm_err: err.to_string(),
            }))
        }
    };

    return Ok(object_bytes);
}
