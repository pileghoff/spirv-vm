use std::collections::HashMap;

use crate::{
    id_types::{BlockId, FunctionId, ValueId},
    instructions::{Instruction, Terminator},
    memory_store::{MemoryStore, mem_read_inner},
    program::Program,
    types::{Pointer, RuntimeScalarValue, RuntimeValue, Storage},
};

#[derive(Debug, Clone)]
pub struct ExecutionContex {
    pub program: Program,

    pub current_block: BlockId,
    pub current_block_index: usize,
    pub function_stack: Vec<(BlockId, usize, Option<ValueId>)>,
    pub function_memory: Vec<MemoryStore>,
    pub values: HashMap<ValueId, RuntimeValue>,

    pub current_line: Option<u32>,
}

pub enum ExecutionNext {
    Terminator(Terminator),
    Instruction(Instruction),
}

#[macro_export]
macro_rules! match_scalar {
    ( $v1:ident, $v2:ident, $op:expr ) => {
        match ($v1, $v2) {
            (RuntimeScalarValue::I8($v1), RuntimeScalarValue::I8($v2)) => {
                Some(Into::<RuntimeScalarValue>::into($op))
            }
            (RuntimeScalarValue::I16($v1), RuntimeScalarValue::I16($v2)) => {
                Some(Into::<RuntimeScalarValue>::into($op))
            }
            (RuntimeScalarValue::I32($v1), RuntimeScalarValue::I32($v2)) => {
                Some(Into::<RuntimeScalarValue>::into($op))
            }
            (RuntimeScalarValue::I64($v1), RuntimeScalarValue::I64($v2)) => {
                Some(Into::<RuntimeScalarValue>::into($op))
            }
            (RuntimeScalarValue::U8($v1), RuntimeScalarValue::U8($v2)) => {
                Some(Into::<RuntimeScalarValue>::into($op))
            }
            (RuntimeScalarValue::U16($v1), RuntimeScalarValue::U16($v2)) => {
                Some(Into::<RuntimeScalarValue>::into($op))
            }
            (RuntimeScalarValue::U32($v1), RuntimeScalarValue::U32($v2)) => {
                Some(Into::<RuntimeScalarValue>::into($op))
            }
            (RuntimeScalarValue::U64($v1), RuntimeScalarValue::U64($v2)) => {
                Some(Into::<RuntimeScalarValue>::into($op))
            }
            _ => None,
        }
    };
}

#[macro_export]
macro_rules! matching_scalar {
    ( $scalar:ident, $op1:ident, $op2:ident ) => {
        (
            RuntimeValue::Scalar(RuntimeScalarValue::$scalar($op1)),
            RuntimeValue::Scalar(RuntimeScalarValue::$scalar($op2)),
        )
    };
}

fn scalar_val_to_i64(value: &RuntimeScalarValue) -> Option<i64> {
    match value {
        RuntimeScalarValue::I8(v) => Some((*v).into()),
        RuntimeScalarValue::I16(v) => Some((*v).into()),
        RuntimeScalarValue::I32(v) => Some((*v).into()),
        RuntimeScalarValue::I64(v) => Some(*v),
        _ => None,
    }
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

impl ExecutionContex {
    pub fn new(program: Program) -> Self {
        let function_memory = vec![MemoryStore::new(Storage::Function)];
        let function_stack = Vec::new();
        let current_block = {
            let f = program.functions.get(&program.entry_point).unwrap();
            *f.blocks.first().unwrap()
        };
        let current_block_index = 0;
        let values = HashMap::new();

        ExecutionContex {
            program,
            current_block,
            current_block_index,
            function_stack,
            function_memory,
            values,
            current_line: None,
        }
    }

    pub fn peek_next_instuction(&self) -> Option<ExecutionNext> {
        if let Some(block) = self.program.blocks.get(&self.current_block) {
            if self.current_block_index < block.instructions.len() {
                Some(ExecutionNext::Instruction(
                    block.instructions[self.current_block_index].clone(),
                ))
            } else {
                Some(ExecutionNext::Terminator(block.terminator.clone()))
            }
        } else {
            None
        }
    }

    pub fn next_instuction(&mut self) -> Option<ExecutionNext> {
        let next = self.peek_next_instuction();
        if let Some(ExecutionNext::Instruction(_)) = next {
            self.current_block_index += 1;
        }
        next
    }

    pub fn push_func(
        &mut self,
        id: &FunctionId,
        args: Vec<ValueId>,
        return_value_id: Option<ValueId>,
    ) {
        let f = self.program.functions.get(id).unwrap();
        self.function_stack.push((
            self.current_block,
            self.current_block_index,
            return_value_id,
        ));
        for (arg_in, arg_out) in f.args.iter().zip(args.iter()) {
            self.values.insert(*arg_in, self.read(arg_out).unwrap());
        }
        self.current_block = *f.blocks.first().unwrap();
        self.current_block_index = 0;
        self.function_memory
            .push(MemoryStore::new(Storage::Function));
    }

    pub fn pop_func(&mut self, out_id: Option<ValueId>) {
        if let Some((b_id, i, r_id)) = self.function_stack.pop() {
            self.function_memory.pop();
            self.current_block = b_id;
            self.current_block_index = i;
            if let Some(r_out_id) = out_id {
                self.values
                    .insert(r_id.unwrap(), self.read(&r_out_id).unwrap().clone());
            }
        } else {
            self.current_block = u32::MAX.into();
        }
    }

    pub fn jump(&mut self, block: BlockId) {
        self.current_block = block;
        self.current_block_index = 0;
    }

    pub fn read(&self, id: &ValueId) -> Option<RuntimeValue> {
        if self.values.contains_key(id) {
            self.values.get(id).cloned()
        } else {
            self.program.values.get(id).cloned()
        }
    }

    pub fn write(&mut self, id: &ValueId, value: RuntimeValue) {
        self.values.insert(*id, value);
    }

    pub fn mem_write(&mut self, ptr: &ValueId, val: &ValueId) {
        let ptr = self.read(ptr).unwrap();
        let val = self.read(val).unwrap();

        match ptr {
            RuntimeValue::Pointer(pointer) => {
                for store in self.function_memory.iter_mut() {
                    if store.id == pointer.storage_id {
                        return store.write(pointer, val);
                    }
                }

                panic!();
            }
            _ => panic!("Incorrect pointer: {:?}", ptr),
        };
    }

    pub fn mem_read(&self, ptr: &ValueId) -> Option<RuntimeValue> {
        let ptr = self.read(ptr).unwrap();

        match ptr {
            RuntimeValue::Pointer(pointer) => {
                for store in self.function_memory.iter() {
                    if store.id == pointer.storage_id {
                        return store.read(pointer);
                    }
                }
                panic!(
                    "Missing id {} in {:?}",
                    pointer.id,
                    self.function_memory.iter().map(|v| v.id)
                );
            }
            _ => panic!("Incorrect pointer: {:?}", ptr),
        }
    }

    pub fn mem_alloc(&mut self, storage: Storage, init: Option<RuntimeValue>) -> RuntimeValue {
        if storage == Storage::Function {
            self.function_memory.last_mut().unwrap().alloc(init).into()
        } else {
            panic!()
        }
    }

    pub fn find_valueid_for_name(&self, name: &str) -> Option<ValueId> {
        self.program
            .values_name
            .iter()
            .filter(|p| p.1 == name)
            .collect::<Vec<_>>()
            .first()
            .map(|p| *p.0)
    }

    pub fn stopped(&self) -> bool {
        self.peek_next_instuction().is_none()
    }

    pub fn step(&mut self) {
        match self.next_instuction() {
            Some(ExecutionNext::Instruction(i)) => match i {
                Instruction::Line(l) => {
                    self.current_line = Some(l);
                }
                Instruction::Call(r_id, f_id, args) => {
                    self.push_func(&f_id, args, r_id);
                }
                Instruction::Store { from, ptr } => {
                    self.mem_write(&ptr, &from);
                }
                Instruction::Load { out, ptr } => {
                    let val = self.mem_read(&ptr).unwrap();
                    self.values.insert(out, val);
                }
                Instruction::CopyInner {
                    out,
                    composite,
                    offsets,
                } => {
                    let composite = self.read(&composite).unwrap();
                    let res = mem_read_inner(offsets.clone(), composite).unwrap();
                    println!("CopyInner %{:?}: {}", out, res.pretty());
                    self.write(&out, res);
                }
                Instruction::CreateInnerPointer { out, base, offsets } => {
                    let (storage_id, base_ptr) = match self.read(&base) {
                        Some(RuntimeValue::Pointer(Pointer {
                            storage_id,
                            id,
                            offsets: _,
                        })) => (storage_id, id),
                        _ => todo!(),
                    };
                    let offsets: Vec<usize> = offsets
                        .iter()
                        .map(|offset_id| self.read(offset_id).unwrap().try_into().unwrap())
                        .collect();
                    let ptr = RuntimeValue::Pointer(Pointer {
                        storage_id,
                        id: base_ptr,
                        offsets,
                    });

                    self.values.insert(out, ptr);
                }
                Instruction::Alloc { out, storage, init } => {
                    if let Some(init) = init {
                        let init = self.read(&init);
                        let ptr = self.mem_alloc(storage, init);
                        self.write(&out, ptr);
                    } else {
                        todo!();
                    }
                }
                Instruction::GreaterThan(v_id, op1, op2) => {
                    let op1: i64 = val_to_i64(&self.read(&op1).unwrap()).unwrap();
                    let op2: i64 = val_to_i64(&self.read(&op2).unwrap()).unwrap();

                    self.values
                        .insert(v_id, RuntimeScalarValue::Bool(op1 > op2).into());
                }
                Instruction::GreaterThanEq(v_id, op1, op2) => {
                    let op1: i64 = val_to_i64(&self.read(&op1).unwrap()).unwrap();
                    let op2: i64 = val_to_i64(&self.read(&op2).unwrap()).unwrap();

                    self.values
                        .insert(v_id, RuntimeScalarValue::Bool(op1 >= op2).into());
                }

                Instruction::LessThan(v_id, op1, op2) => {
                    let op1: i64 = val_to_i64(&self.read(&op1).unwrap()).unwrap();
                    let op2: i64 = val_to_i64(&self.read(&op2).unwrap()).unwrap();

                    self.values
                        .insert(v_id, RuntimeScalarValue::Bool(op1 < op2).into());
                }
                Instruction::LessThanEq(v_id, op1, op2) => {
                    let op1: i64 = val_to_i64(&self.read(&op1).unwrap()).unwrap();
                    let op2: i64 = val_to_i64(&self.read(&op2).unwrap()).unwrap();

                    self.values
                        .insert(v_id, RuntimeScalarValue::Bool(op1 <= op2).into());
                }

                Instruction::Equal(v_id, op1, op2) => {
                    println!("{:?}", self);
                    println!("{:?}", op1);
                    let op1 = self.read(&op1).unwrap();
                    let op2 = self.read(&op2).unwrap();
                    let res: RuntimeValue = match (op1.clone(), op2.clone()) {
                        (RuntimeValue::Scalar(v1), RuntimeValue::Scalar(v2)) => (v1 == v2).into(),
                        (
                            RuntimeValue::Vec {
                                lenght: _,
                                contents: op1,
                            },
                            RuntimeValue::Vec {
                                lenght,
                                contents: op2,
                            },
                        ) => op1
                            .iter()
                            .zip(op2.iter())
                            .map(|(v1, v2)| v1 == v2)
                            .collect::<Vec<_>>()
                            .into(),

                        _ => panic!("Mismatched types {:?}, {:?}", op1, op2),
                    };

                    self.values.insert(v_id, res);
                }
                Instruction::Add(v_id, op1, op2) => {
                    let res: RuntimeValue =
                        match (self.read(&op1).unwrap(), self.read(&op2).unwrap()) {
                            (RuntimeValue::Scalar(op1), RuntimeValue::Scalar(op2)) => {
                                match_scalar!(op1, op2, op1.wrapping_add(op2))
                                    .unwrap()
                                    .into()
                            }
                            (
                                RuntimeValue::Vec {
                                    lenght: _,
                                    contents: op1,
                                },
                                RuntimeValue::Vec {
                                    lenght: _,
                                    contents: op2,
                                },
                            ) => {
                                let res = op1
                                    .iter()
                                    .zip(op2.iter())
                                    .map(|(v1, v2)| {
                                        match_scalar!(v1, v2, v1.wrapping_add(*v2)).unwrap()
                                    })
                                    .collect::<Vec<RuntimeScalarValue>>();

                                RuntimeValue::Vec {
                                    lenght: res.len(),
                                    contents: res,
                                }
                            }
                            _ => panic!(),
                        };

                    self.values.insert(v_id, res);
                }
                Instruction::Sub(v_id, op1, op2) => {
                    let res: RuntimeValue =
                        match (self.read(&op1).unwrap(), self.read(&op2).unwrap()) {
                            (RuntimeValue::Scalar(op1), RuntimeValue::Scalar(op2)) => {
                                match_scalar!(op1, op2, op1.wrapping_sub(op2))
                                    .unwrap()
                                    .into()
                            }
                            (
                                RuntimeValue::Vec {
                                    lenght: _,
                                    contents: op1,
                                },
                                RuntimeValue::Vec {
                                    lenght: _,
                                    contents: op2,
                                },
                            ) => {
                                let res = op1
                                    .iter()
                                    .zip(op2.iter())
                                    .map(|(v1, v2)| {
                                        match_scalar!(v1, v2, v1.wrapping_sub(*v2)).unwrap()
                                    })
                                    .collect::<Vec<RuntimeScalarValue>>();

                                RuntimeValue::Vec {
                                    lenght: res.len(),
                                    contents: res,
                                }
                            }
                            _ => panic!(),
                        };

                    self.values.insert(v_id, res);
                }
                Instruction::Select {
                    out,
                    condition,
                    op1,
                    op2,
                } => {
                    let condition = self.read(&condition).unwrap();
                    let op1 = self.read(&op1).unwrap();
                    let op2 = self.read(&op2).unwrap();
                    let res = match condition {
                        RuntimeValue::Scalar(RuntimeScalarValue::Bool(cond)) => {
                            if cond {
                                op1
                            } else {
                                op2
                            }
                        }
                        _ => panic!(
                            "Mismatched types for Selcet: {:?}, {:?}, {:?}",
                            condition, op1, op2
                        ),
                    };
                    self.write(&out, res);
                }
                Instruction::VecAllTrue(v_id, vector) => {
                    let vector = self.read(&vector).unwrap();
                    if let RuntimeValue::Vec { lenght, contents } = vector {
                        let res = contents.iter().all(|v| {
                            if let RuntimeScalarValue::Bool(v) = v {
                                *v
                            } else {
                                panic!("Vector contains non-bool value: {:?}", v);
                            }
                        });
                        self.write(&v_id, res.into());
                    } else {
                        panic!("Bad type for VecAllTrue: {:?}", vector);
                    }
                }
                Instruction::CreateStruct { out, members } => {
                    self.write(
                        &out,
                        RuntimeValue::Struct {
                            members: members.iter().map(|v| self.read(v).unwrap()).collect(),
                        },
                    );
                }
                Instruction::CreateVec { out, members } => {
                    self.write(
                        &out,
                        RuntimeValue::Vec {
                            lenght: members.len(),
                            contents: members
                                .iter()
                                .map(|v| {
                                    TryInto::<RuntimeScalarValue>::try_into(self.read(v).unwrap())
                                        .unwrap()
                                })
                                .collect(),
                        },
                    );
                }
            },
            Some(ExecutionNext::Terminator(t)) => match t {
                Terminator::Jump(b_id) => {
                    self.jump(b_id);
                }
                Terminator::Branch {
                    condition,
                    then_block,
                    else_block,
                } => match self.read(&condition) {
                    Some(RuntimeValue::Scalar(RuntimeScalarValue::Bool(cond))) => {
                        if cond {
                            self.jump(then_block);
                        } else {
                            self.jump(else_block);
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
                    self.pop_func(out_id);
                }
            },
            None => return,
        }
    }
}
