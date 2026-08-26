use std::collections::HashMap;

use crate::{
    id_types::{BlockId, FunctionId, ValueId},
    instructions::{Instruction, Terminator},
    memory_store::MemoryStore,
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
                Instruction::IGreaterThan(v_id, op1, op2) => {
                    let op1: i64 = val_to_i64(&self.read(&op1).unwrap()).unwrap();
                    let op2: i64 = val_to_i64(&self.read(&op2).unwrap()).unwrap();

                    self.values
                        .insert(v_id, RuntimeScalarValue::Bool(op1 > op2).into());
                }
                Instruction::IGreaterThanEq(v_id, op1, op2) => {
                    let op1: i64 = val_to_i64(&self.read(&op1).unwrap()).unwrap();
                    let op2: i64 = val_to_i64(&self.read(&op2).unwrap()).unwrap();

                    self.values
                        .insert(v_id, RuntimeScalarValue::Bool(op1 >= op2).into());
                }

                Instruction::ILessThan(v_id, op1, op2) => {
                    let op1: i64 = val_to_i64(&self.read(&op1).unwrap()).unwrap();
                    let op2: i64 = val_to_i64(&self.read(&op2).unwrap()).unwrap();

                    self.values
                        .insert(v_id, RuntimeScalarValue::Bool(op1 < op2).into());
                }
                Instruction::ILessThanEq(v_id, op1, op2) => {
                    let op1: i64 = val_to_i64(&self.read(&op1).unwrap()).unwrap();
                    let op2: i64 = val_to_i64(&self.read(&op2).unwrap()).unwrap();

                    self.values
                        .insert(v_id, RuntimeScalarValue::Bool(op1 <= op2).into());
                }

                Instruction::IEqual(v_id, op1, op2) => {
                    let op1: i64 = val_to_i64(&self.read(&op1).unwrap()).unwrap();
                    let op2: i64 = val_to_i64(&self.read(&op2).unwrap()).unwrap();

                    self.values
                        .insert(v_id, RuntimeScalarValue::Bool(op1 == op2).into());
                }
                Instruction::IAdd(v_id, op1, op2) => {
                    let res = match (self.read(&op1).unwrap(), self.read(&op2).unwrap()) {
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

                    self.values.insert(v_id, res);
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
