use crate::id_types::{BlockId, FunctionId, TypeId, ValueId};
use crate::instructions::Instruction;
use crate::program::{Program, Terminator};
use std::{collections::HashMap, thread::sleep, time::Duration};

use crate::types::*;

pub fn run(mut program: Program) {
    println!("{:?}", program);
    println!("Before:");
    program.vals();

    program.function_memory = vec![HashMap::new()];
    program.function_stack = Vec::new();
    let mut current_block: BlockId = {
        let f = program.functions.get(&program.entry_point).unwrap();
        *f.blocks.first().unwrap()
    };
    let mut current_block_index: usize = 0;

    loop {
        // Execute block
        let block = program.blocks.get(&current_block).unwrap();
        if current_block_index < block.instructions.len() {
            sleep(Duration::from_millis(100));
            let i = block.instructions[current_block_index].clone();
            current_block_index += 1;
            println!("{:?} [{current_block}]", i);
            match i {
                Instruction::Call(r_id, f_id, args) => {
                    let f = program.functions.get(&f_id).unwrap();
                    program
                        .function_stack
                        .push((current_block, current_block_index, r_id.clone()));
                    for (arg_in, arg_out) in f.args.iter().zip(args.iter()) {
                        let v = program.values.get(arg_out).unwrap();
                        program.values.insert(*arg_in, v.clone());
                    }
                    current_block = *f.blocks.first().unwrap();
                    current_block_index = 0;
                    println!("Call function {f_id} -> {current_block}");
                }
                Instruction::Store { from, ptr } => {
                    let ptr = program.values.get(&ptr).unwrap();
                    match ptr {
                        RuntimeValue::Pointer {
                            storage: Storage::Function,
                            id,
                        } => {
                            let mem: &mut HashMap<ValueId, RuntimeValue> =
                                program.function_memory.last_mut().unwrap();
                            let val = program.values.get(&from).unwrap();
                            mem.insert(*id, val.clone());
                        }
                        _ => todo!(),
                    }
                }
                Instruction::Load { out, ptr } => {
                    let ptr = program.values.get(&ptr).unwrap();
                    match ptr {
                        RuntimeValue::Pointer { storage, id } => match storage {
                            Storage::Function => {
                                let mem: &HashMap<ValueId, RuntimeValue> =
                                    program.function_memory.last().unwrap();
                                let v_id: ValueId = id.clone();
                                let v: RuntimeValue = mem.get(&v_id).unwrap().clone();
                                program.values.insert(out, v);
                            }
                        },
                        _ => todo!(),
                    }
                }
                Instruction::Alloc {
                    out,
                    storage,
                    t_id,
                    init,
                } => match storage {
                    Storage::Function => {
                        let mem: &mut HashMap<ValueId, RuntimeValue> =
                            program.function_memory.last_mut().unwrap();
                        if let Some(init) = init {
                            let value: RuntimeValue = program.values.get(&init).unwrap().clone();
                            let v_id: ValueId = rand::random::<u32>().into();

                            mem.insert(v_id, value);
                            program
                                .values
                                .insert(out, RuntimeValue::Pointer { storage, id: v_id });
                        } else {
                            todo!();
                        }
                    }
                },
                Instruction::IEqual(v_id, op1, op2) => {
                    let op1: i64 = match program.values.get(&op1).unwrap() {
                        RuntimeValue::I8(v) => (*v).into(),
                        RuntimeValue::I16(v) => (*v).into(),
                        RuntimeValue::I32(v) => (*v).into(),
                        RuntimeValue::I64(v) => (*v).into(),
                        _ => todo!(),
                    };

                    let op2: i64 = match program.values.get(&op2).unwrap() {
                        RuntimeValue::I8(v) => (*v).into(),
                        RuntimeValue::I16(v) => (*v).into(),
                        RuntimeValue::I32(v) => (*v).into(),
                        RuntimeValue::I64(v) => (*v).into(),
                        _ => todo!(),
                    };

                    program.values.insert(v_id, RuntimeValue::Bool(op1 == op2));
                }
                Instruction::IAdd(v_id, t_id, op1, op2) => {
                    let res = match (
                        program.values.get(&op1).unwrap(),
                        program.values.get(&op2).unwrap(),
                    ) {
                        (RuntimeValue::I8(v1), RuntimeValue::I8(v2)) => {
                            RuntimeValue::I8(v1.wrapping_add(*v2))
                        }
                        (RuntimeValue::I16(v1), RuntimeValue::I16(v2)) => {
                            RuntimeValue::I16(v1.wrapping_add(*v2))
                        }
                        (RuntimeValue::I32(v1), RuntimeValue::I32(v2)) => {
                            RuntimeValue::I32(v1.wrapping_add(*v2))
                        }
                        (RuntimeValue::I64(v1), RuntimeValue::I64(v2)) => {
                            RuntimeValue::I64(v1.wrapping_add(*v2))
                        }
                        (bad1, bad2) => panic!("Failed to add {:?} and {:?}", bad1, bad2),
                    };

                    program.values.insert(v_id, res);
                }
            }
        } else {
            println!("{:?}", block.terminator);
            match &block.terminator {
                Terminator::Jump(b_id) => {
                    current_block = *b_id;
                    current_block_index = 0;
                }
                Terminator::Branch {
                    condition,
                    then_block,
                    else_block,
                } => match program.values.get(condition) {
                    Some(RuntimeValue::Bool(cond)) => {
                        if *cond {
                            current_block = *then_block;
                        } else {
                            current_block = *else_block;
                        }
                        current_block_index = 0;
                    }
                    _ => panic!("{:?}", condition),
                },
                Terminator::Switch {
                    selector,
                    cases,
                    default,
                } => todo!(),
                Terminator::Return(r_out_id) => {
                    if let Some((b_id, i, r_id)) = program.function_stack.pop() {
                        current_block = b_id;
                        current_block_index = i + 1;
                        if let Some(r_out_id) = r_out_id {
                            program.vals();

                            program.values.insert(
                                r_id.unwrap(),
                                program.values.clone().get(r_out_id).unwrap().clone(),
                            );
                        }
                    } else {
                        break;
                    }
                }
            };
        }
    }

    println!("");
    println!("After:");
    program.vals();
}
