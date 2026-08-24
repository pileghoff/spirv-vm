use crate::execution_context::ExecutionNext;
use crate::instructions::{Instruction, Terminator};

use crate::program::Program;

use crate::{execution_context::ExecutionContex, types::*};

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

pub fn run(program: Program) -> ExecutionContex {
    println!("{:?}", program);
    println!("Before:");
    program.vals();
    let mut context = ExecutionContex::new(program);

    loop {
        // Execute block
        match context.next() {
            Some(ExecutionNext::Instruction(i)) => {
                println!("{:?} [{}]", i, context.current_block);
                match i {
                    Instruction::Call(r_id, f_id, args) => {
                        context.push_func(&f_id, args, r_id);
                        println!("Call function {f_id} -> {}", context.current_block);
                    }
                    Instruction::Store { from, ptr } => {
                        context.mem_write(&ptr, &from);
                    }
                    Instruction::Load { out, ptr } => {
                        let val = context.mem_read(&ptr).unwrap();
                        context.values.insert(out, val);
                    }
                    Instruction::CreateInnerPointer { out, base, offsets } => {
                        let (storage_id, base_ptr) = match context.read(&base) {
                            Some(RuntimeValue::Pointer(Pointer {
                                storage_id,
                                id,
                                offsets: _,
                            })) => (storage_id, id),
                            _ => todo!(),
                        };
                        let offsets: Vec<usize> = offsets
                            .iter()
                            .map(|offset_id| context.read(offset_id).unwrap().try_into().unwrap())
                            .collect();
                        let ptr = RuntimeValue::Pointer(Pointer {
                            storage_id,
                            id: base_ptr,
                            offsets,
                        });

                        context.values.insert(out, ptr);
                    }
                    Instruction::Alloc { out, storage, init } => {
                        if let Some(init) = init {
                            let init = context.read(&init);
                            let ptr = context.mem_alloc(storage, init);
                            context.write(&out, ptr);
                        } else {
                            todo!();
                        }
                    }
                    Instruction::IGreaterThan(v_id, op1, op2) => {
                        let op1: i64 = val_to_i64(&context.read(&op1).unwrap()).unwrap();
                        let op2: i64 = val_to_i64(&context.read(&op2).unwrap()).unwrap();

                        context
                            .values
                            .insert(v_id, RuntimeScalarValue::Bool(op1 > op2).into());
                    }
                    Instruction::IGreaterThanEq(v_id, op1, op2) => {
                        let op1: i64 = val_to_i64(&context.read(&op1).unwrap()).unwrap();
                        let op2: i64 = val_to_i64(&context.read(&op2).unwrap()).unwrap();

                        context
                            .values
                            .insert(v_id, RuntimeScalarValue::Bool(op1 >= op2).into());
                    }

                    Instruction::ILessThan(v_id, op1, op2) => {
                        let op1: i64 = val_to_i64(&context.read(&op1).unwrap()).unwrap();
                        let op2: i64 = val_to_i64(&context.read(&op2).unwrap()).unwrap();

                        context
                            .values
                            .insert(v_id, RuntimeScalarValue::Bool(op1 < op2).into());
                    }
                    Instruction::ILessThanEq(v_id, op1, op2) => {
                        let op1: i64 = val_to_i64(&context.read(&op1).unwrap()).unwrap();
                        let op2: i64 = val_to_i64(&context.read(&op2).unwrap()).unwrap();

                        context
                            .values
                            .insert(v_id, RuntimeScalarValue::Bool(op1 <= op2).into());
                    }

                    Instruction::IEqual(v_id, op1, op2) => {
                        let op1: i64 = val_to_i64(&context.read(&op1).unwrap()).unwrap();
                        let op2: i64 = val_to_i64(&context.read(&op2).unwrap()).unwrap();

                        context
                            .values
                            .insert(v_id, RuntimeScalarValue::Bool(op1 == op2).into());
                    }
                    Instruction::IAdd(v_id, op1, op2) => {
                        let res = match (context.read(&op1).unwrap(), context.read(&op2).unwrap()) {
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

                        context.values.insert(v_id, res);
                    }
                }
            }
            Some(ExecutionNext::Terminator(t)) => {
                println!("{:?}", t);
                match t {
                    Terminator::Jump(b_id) => {
                        context.jump(b_id);
                    }
                    Terminator::Branch {
                        condition,
                        then_block,
                        else_block,
                    } => match context.read(&condition) {
                        Some(RuntimeValue::Scalar(RuntimeScalarValue::Bool(cond))) => {
                            if cond {
                                context.jump(then_block);
                            } else {
                                context.jump(else_block);
                            }
                        }
                        _ => panic!("{:?}", condition),
                    },
                    Terminator::Switch {
                        selector: _,
                        cases: _,
                        default: _,
                    } => todo!(),
                    Terminator::Return(out_id) => {
                        context.pop_func(out_id);
                    }
                }
            }
            None => break,
        }
    }

    println!();
    println!("After:");
    context.vals();

    context
}
