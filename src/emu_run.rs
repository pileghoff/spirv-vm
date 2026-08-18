use crate::id_types::{BlockId, FunctionId, TypeId, ValueId};
use std::{thread::sleep, time::Duration};

use crate::emu_types::*;

pub fn run(mut program: Program) {
    println!("{:?}", program);
    println!("Before:");
    program.vals();

    let mut function_stack: Vec<(BlockId, usize, Option<ValueId>)> = vec![];
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
                Instruction::Unknown => todo!(),
                Instruction::Call(r_id, f_id, args) => {
                    let f = program.functions.get(&f_id).unwrap();
                    function_stack.push((current_block, current_block_index, r_id.clone()));
                    for (arg_in, arg_out) in f.args.iter().zip(args.iter()) {
                        let v = program.values.get(arg_out).unwrap();
                        program.values.insert(*arg_in, v.clone());
                    }
                    current_block = *f.blocks.first().unwrap();
                    current_block_index = 0;
                    println!("Call function {f_id} -> {current_block}");
                }
                Instruction::IAdd(v_id, t_id, op1, op2) => {
                    let op1: i32 = i32::from_ne_bytes(
                        program
                            .values
                            .get(&op1)
                            .unwrap()
                            .value
                            .clone()
                            .try_into()
                            .unwrap(),
                    );

                    let op2: i32 = i32::from_ne_bytes(
                        program
                            .values
                            .get(&op2)
                            .unwrap()
                            .value
                            .clone()
                            .try_into()
                            .unwrap(),
                    );
                    let t = program.typemap.get(&t_id).unwrap();

                    program.values.insert(
                        v_id,
                        RuntimeValue {
                            value_type: t.clone(),
                            value: (op1 + op2).to_ne_bytes().to_vec(),
                        },
                    );
                }
            }
        } else {
            println!("{:?}", block.terminator);
            match &block.terminator {
                Terminator::Jump(b_id) => current_block = *b_id,
                Terminator::Branch {
                    condition,
                    then_block,
                    else_block,
                } => todo!(),
                Terminator::Switch {
                    selector,
                    cases,
                    default,
                } => todo!(),
                Terminator::Return(r_out_id) => {
                    if let Some((b_id, i, r_id)) = function_stack.pop() {
                        current_block = b_id;
                        current_block_index = i + 1;
                        if let Some(r_out_id) = r_out_id {
                            println!("Try to read {:?} into {}", r_id, r_out_id);
                            println!("");
                            println!("Current:");
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
