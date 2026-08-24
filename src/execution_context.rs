use std::collections::HashMap;

use crate::{
    id_types::{BlockId, FunctionId, MemValueId, TypeId, ValueId},
    instructions::Instruction,
    memory_store::MemoryStore,
    program::{Program, Terminator},
    types::{Pointer, RuntimeValue, Storage, Type},
};

pub struct ExecutionContex {
    pub program: Program,

    pub current_block: BlockId,
    pub current_block_index: usize,
    pub function_stack: Vec<(BlockId, usize, Option<ValueId>)>,
    pub function_memory: Vec<MemoryStore>,
    pub values: HashMap<ValueId, RuntimeValue>,
}

pub enum ExecutionNext {
    Terminator(Terminator),
    Instruction(Instruction),
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
        }
    }

    pub fn next(&mut self) -> Option<ExecutionNext> {
        if let Some(block) = self.program.blocks.get(&self.current_block) {
            if self.current_block_index < block.instructions.len() {
                self.current_block_index += 1;
                Some(ExecutionNext::Instruction(
                    block.instructions[self.current_block_index - 1].clone(),
                ))
            } else {
                Some(ExecutionNext::Terminator(block.terminator.clone()))
            }
        } else {
            None
        }
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
            return_value_id.clone(),
        ));
        for (arg_in, arg_out) in f.args.iter().zip(args.iter()) {
            println!("%{arg_in}: %{arg_out}");
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
                println!("Return %{r_out_id} into %{}", r_id.unwrap());
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

    pub fn vals(&self) {
        for (v_id, v) in self.program.values.iter() {
            if let Some(name) = self.program.values_name.get(v_id) {
                println!("{name}[%{v_id}]: {}", v.pretty());
            } else {
                println!("%{v_id}: {}", v.pretty());
            }
        }

        for (v_id, v) in self.values.iter() {
            if let Some(name) = self.program.values_name.get(v_id) {
                println!("{name}[%{v_id}]: {}", v.pretty());
            } else {
                println!("%{v_id}: {}", v.pretty());
            }
        }
    }

    pub fn read(&self, id: &ValueId) -> Option<RuntimeValue> {
        if self.values.contains_key(id) {
            self.values.get(id).cloned()
        } else {
            self.program.values.get(id).cloned()
        }
    }

    pub fn write(&mut self, id: &ValueId, value: RuntimeValue) {
        self.values.insert(id.clone(), value);
    }

    pub fn mem_write(&mut self, ptr: &ValueId, val: &ValueId) {
        let ptr = self.read(ptr).unwrap();
        let val = self.read(val).unwrap();
        self.vals();

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
}
