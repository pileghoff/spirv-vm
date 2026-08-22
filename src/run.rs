use crate::id_types::{BlockId, FunctionId, MemValueId, TypeId, ValueId};
use crate::instructions::Instruction;
use crate::memory_store::MemoryStore;
use crate::program::{Program, ProgramNext, Terminator};
use std::{collections::HashMap, thread::sleep, time::Duration};

use crate::types::*;

#[macro_export]
macro_rules! matching_scalar {
    ( $scalar:ident, $op1:ident, $op2:ident ) => {
        (
            RuntimeValue::Scalar(RuntimeScalarValue::$scalar($op1)),
            RuntimeValue::Scalar(RuntimeScalarValue::$scalar($op2)),
        )
    };
}

fn val_to_i64(value: &RuntimeValue) -> Option<i64> {
    match value {
        RuntimeValue::Scalar(RuntimeScalarValue::I8(v)) => Some((*v).into()),
        RuntimeValue::Scalar(RuntimeScalarValue::I16(v)) => Some((*v).into()),
        RuntimeValue::Scalar(RuntimeScalarValue::I32(v)) => Some((*v).into()),
        RuntimeValue::Scalar(RuntimeScalarValue::I64(v)) => Some(*v),
        _ => None,
    }
}

pub fn run(program: &mut Program) {
    println!("{:?}", program);
    println!("Before:");
    program.vals();

    program.function_memory = vec![MemoryStore::new(Storage::Function)];
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
                    Instruction::CreateInnerPointer { out, base, offsets } => {
                        let (storage_id, base_ptr) = match program.read(&base) {
                            Some(RuntimeValue::Pointer(Pointer {
                                storage_id,
                                id,
                                offsets,
                            })) => (storage_id, id),
                            _ => todo!(),
                        };
                        let offsets: Vec<usize> = offsets
                            .iter()
                            .map(|offset_id| program.read(offset_id).unwrap().try_into().unwrap())
                            .collect();
                        let base_val = program.mem_read(&base).unwrap();
                        match base_val {
                            RuntimeValue::Vec { lenght, contents } => {
                                let ptr = RuntimeValue::Pointer(Pointer {
                                    storage_id,
                                    id: base_ptr,
                                    offsets,
                                });

                                println!("----");
                                println!("Create composite pointer %{} {:?}", out, ptr);
                                println!("----");
                                program.values.insert(out, ptr);
                            }
                            RuntimeValue::Pointer(Pointer {
                                storage_id: _,
                                id: _,
                                offsets: _,
                            })
                            | RuntimeValue::Null
                            | RuntimeValue::Void
                            | RuntimeValue::Scalar(_) => {
                                panic!("Unsupported type for access chain: {:?}", base)
                            }
                        }
                    }
                    Instruction::Alloc { out, storage, init } => {
                        if let Some(init) = init {
                            let init = program.read(&init);
                            let ptr = program.mem_alloc(storage, init);
                            program.write(&out, ptr);
                        } else {
                            todo!();
                        }
                    }
                    Instruction::IGreaterThan(v_id, op1, op2) => {
                        let op1: i64 = val_to_i64(&program.read(&op1).unwrap()).unwrap();
                        let op2: i64 = val_to_i64(&program.read(&op2).unwrap()).unwrap();

                        program
                            .values
                            .insert(v_id, RuntimeScalarValue::Bool(op1 > op2).into());
                    }
                    Instruction::IGreaterThanEq(v_id, op1, op2) => {
                        let op1: i64 = val_to_i64(&program.read(&op1).unwrap()).unwrap();
                        let op2: i64 = val_to_i64(&program.read(&op2).unwrap()).unwrap();

                        program
                            .values
                            .insert(v_id, RuntimeScalarValue::Bool(op1 >= op2).into());
                    }

                    Instruction::ILessThan(v_id, op1, op2) => {
                        let op1: i64 = val_to_i64(&program.read(&op1).unwrap()).unwrap();
                        let op2: i64 = val_to_i64(&program.read(&op2).unwrap()).unwrap();

                        program
                            .values
                            .insert(v_id, RuntimeScalarValue::Bool(op1 < op2).into());
                    }
                    Instruction::ILessThanEq(v_id, op1, op2) => {
                        let op1: i64 = val_to_i64(&program.read(&op1).unwrap()).unwrap();
                        let op2: i64 = val_to_i64(&program.read(&op2).unwrap()).unwrap();

                        program
                            .values
                            .insert(v_id, RuntimeScalarValue::Bool(op1 <= op2).into());
                    }

                    Instruction::IEqual(v_id, op1, op2) => {
                        let op1: i64 = val_to_i64(&program.read(&op1).unwrap()).unwrap();
                        let op2: i64 = val_to_i64(&program.read(&op2).unwrap()).unwrap();

                        program
                            .values
                            .insert(v_id, RuntimeScalarValue::Bool(op1 == op2).into());
                    }
                    Instruction::IAdd(v_id, op1, op2) => {
                        let res = match (program.read(&op1).unwrap(), program.read(&op2).unwrap()) {
                            matching_scalar!(U8, op1, op2) => op1.wrapping_add(op2).into(),
                            matching_scalar!(U16, op1, op2) => op1.wrapping_add(op2).into(),
                            matching_scalar!(U32, op1, op2) => op1.wrapping_add(op2).into(),
                            matching_scalar!(U64, op1, op2) => op1.wrapping_add(op2).into(),
                            matching_scalar!(I8, op1, op2) => op1.wrapping_add(op2).into(),
                            matching_scalar!(I16, op1, op2) => op1.wrapping_add(op2).into(),
                            matching_scalar!(I32, op1, op2) => op1.wrapping_add(op2).into(),
                            matching_scalar!(I64, op1, op2) => op1.wrapping_add(op2).into(),
                            (bad1, bad2) => panic!("Failed to add {:?} and {:?}", bad1, bad2),
                        };

                        program.values.insert(v_id, res);
                    }
                }
            }
            Some(ProgramNext::Terminator(t)) => {
                println!("{:?}", t);
                match t {
                    Terminator::Jump(b_id) => {
                        program.jump(b_id);
                    }
                    Terminator::Branch {
                        condition,
                        then_block,
                        else_block,
                    } => match program.read(&condition) {
                        Some(RuntimeValue::Scalar(RuntimeScalarValue::Bool(cond))) => {
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
                }
            }
            None => break,
        }
    }

    println!("");
    println!("After:");
    program.vals();
}
