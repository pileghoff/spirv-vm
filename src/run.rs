use crate::id_types::{BlockId, FunctionId, TypeId, ValueId};
use crate::instructions::Instruction;
use crate::program::{Program, ProgramNext, Terminator};
use std::{collections::HashMap, thread::sleep, time::Duration};

use crate::types::*;

pub fn run(mut program: Program) {
    println!("{:?}", program);
    println!("Before:");
    program.vals();

    program.function_memory = vec![HashMap::new()];
    program.function_stack = Vec::new();
    program.current_block = {
        let f = program.functions.get(&program.entry_point).unwrap();
        *f.blocks.first().unwrap()
    };
    program.current_block_index = 0;

    loop {
        // Execute block
        match program.next() {
            Some(ProgramNext::Instruction(i)) => {
                sleep(Duration::from_millis(100));
                println!("{:?} [{}]", i, program.current_block);
                match i {
                    Instruction::Call(r_id, f_id, args) => {
                        program.push_func(&f_id, args, r_id);
                        println!("Call function {f_id} -> {}", program.current_block);
                    }
                    Instruction::Store { from, ptr } => {
                        program.mem_write(&ptr, &from);
                    }
                    Instruction::Load { out, ptr } => {
                        let val = program.mem_read(&ptr).unwrap();
                        program.values.insert(out, val);
                    }
                    Instruction::Alloc { out, storage, init } => {
                        if let Some(init) = init {
                            // We use a random int as the valueid for now.We should really have a
                            // counter in the program
                            let v_id: ValueId = rand::random::<u32>().into();
                            let ptr = RuntimeValue::Pointer { storage, id: v_id };
                            // Out is where the pointer will be stored.
                            program.write(&out, ptr);
                            program.mem_write(&out, &init);
                        } else {
                            todo!();
                        }
                    }
                    Instruction::IEqual(v_id, op1, op2) => {
                        let op1: i64 = match program.read(&op1).unwrap() {
                            RuntimeValue::I8(v) => v.into(),
                            RuntimeValue::I16(v) => v.into(),
                            RuntimeValue::I32(v) => v.into(),
                            RuntimeValue::I64(v) => v,
                            _ => todo!(),
                        };

                        let op2: i64 = match program.read(&op2).unwrap() {
                            RuntimeValue::I8(v) => v.into(),
                            RuntimeValue::I16(v) => v.into(),
                            RuntimeValue::I32(v) => v.into(),
                            RuntimeValue::I64(v) => v,
                            _ => todo!(),
                        };

                        program.values.insert(v_id, RuntimeValue::Bool(op1 == op2));
                    }
                    Instruction::IAdd(v_id, op1, op2) => {
                        let res = match (program.read(&op1).unwrap(), program.read(&op2).unwrap()) {
                            (RuntimeValue::I8(v1), RuntimeValue::I8(v2)) => {
                                RuntimeValue::I8(v1.wrapping_add(v2))
                            }
                            (RuntimeValue::I16(v1), RuntimeValue::I16(v2)) => {
                                RuntimeValue::I16(v1.wrapping_add(v2))
                            }
                            (RuntimeValue::I32(v1), RuntimeValue::I32(v2)) => {
                                RuntimeValue::I32(v1.wrapping_add(v2))
                            }
                            (RuntimeValue::I64(v1), RuntimeValue::I64(v2)) => {
                                RuntimeValue::I64(v1.wrapping_add(v2))
                            }
                            (bad1, bad2) => panic!("Failed to add {:?} and {:?}", bad1, bad2),
                        };

                        program.values.insert(v_id, res);
                    }
                }
            }
            Some(ProgramNext::Terminator(t)) => match t {
                Terminator::Jump(b_id) => {
                    program.jump(b_id);
                }
                Terminator::Branch {
                    condition,
                    then_block,
                    else_block,
                } => match program.read(&condition) {
                    Some(RuntimeValue::Bool(cond)) => {
                        if cond {
                            program.jump(then_block);
                        } else {
                            program.jump(else_block);
                        }
                    }
                    _ => panic!("{:?}", condition),
                },
                Terminator::Switch {
                    selector,
                    cases,
                    default,
                } => todo!(),
                Terminator::Return(out_id) => {
                    program.pop_func(out_id);
                }
            },
            None => break,
        }
    }

    println!("");
    println!("After:");
    program.vals();
}
