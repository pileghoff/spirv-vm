use std::collections::HashMap;

use crate::{
    id_types::{BlockId, FunctionId, TypeId, ValueId},
    instructions::Instruction,
    types::{RuntimeValue, Storage, Type},
};

#[derive(Debug, Clone)]
pub struct Block {
    pub instructions: Vec<Instruction>,
    pub terminator: Terminator,
}

#[derive(Debug, Clone)]
pub enum Terminator {
    Jump(BlockId),
    Branch {
        condition: ValueId,
        then_block: BlockId,
        else_block: BlockId,
    },
    Switch {
        selector: ValueId,
        cases: Vec<(i32, BlockId)>,
        default: BlockId,
    },
    Return(Option<ValueId>),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub args: Vec<ValueId>,
    pub blocks: Vec<BlockId>,
}

#[derive(Debug, Clone, Default)]
pub struct Program {
    pub entry_point: FunctionId,
    pub functions: HashMap<FunctionId, Function>,

    pub current_block: BlockId,
    pub current_block_index: usize,
    pub function_stack: Vec<(BlockId, usize, Option<ValueId>)>,
    pub function_memory: Vec<HashMap<ValueId, RuntimeValue>>,
    pub typemap: HashMap<TypeId, Type>,
    pub blocks: HashMap<BlockId, Block>,
    pub values: HashMap<ValueId, RuntimeValue>,
    pub values_name: HashMap<ValueId, String>,
}

pub enum ProgramNext {
    Terminator(Terminator),
    Instruction(Instruction),
}

impl Program {
    pub fn next(&mut self) -> Option<ProgramNext> {
        if let Some(block) = self.blocks.get(&self.current_block) {
            if self.current_block_index < block.instructions.len() {
                self.current_block_index += 1;
                Some(ProgramNext::Instruction(
                    block.instructions[self.current_block_index - 1].clone(),
                ))
            } else {
                Some(ProgramNext::Terminator(block.terminator.clone()))
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
        let f = self.functions.get(id).unwrap();
        self.function_stack.push((
            self.current_block,
            self.current_block_index,
            return_value_id.clone(),
        ));
        for (arg_in, arg_out) in f.args.iter().zip(args.iter()) {
            self.values.insert(*arg_in, self.read(arg_out).unwrap());
        }
        self.current_block = *f.blocks.first().unwrap();
        self.current_block_index = 0;
    }

    pub fn pop_func(&mut self, out_id: Option<ValueId>) {
        if let Some((b_id, i, r_id)) = self.function_stack.pop() {
            self.current_block = b_id;
            self.current_block_index = i + 1;
            if let Some(r_out_id) = out_id {
                self.values.insert(
                    r_id.unwrap(),
                    self.values.clone().get(&r_out_id).unwrap().clone(),
                );
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
        for (v_id, v) in self.values.iter() {
            if let Some(name) = self.values_name.get(v_id) {
                println!("%{name}: {}", v.pretty(self));
            } else {
                println!("%{v_id}: {}", v.pretty(self));
            }
        }
    }

    pub fn read(&self, id: &ValueId) -> Option<RuntimeValue> {
        self.values.get(id).cloned()
    }

    pub fn write(&mut self, id: &ValueId, value: RuntimeValue) {
        self.values.insert(id.clone(), value);
    }

    pub fn mem_write(&mut self, ptr: &ValueId, val: &ValueId) {
        let ptr = self.read(ptr).unwrap();
        let val = self.read(val).unwrap();

        match ptr {
            RuntimeValue::Pointer {
                storage: Storage::Function,
                id,
            } => {
                self.function_memory.last_mut().unwrap().insert(id, val);
            }
            _ => panic!("Incorrect pointer: {:?}", ptr),
        };
    }

    pub fn mem_read(&self, ptr: &ValueId) -> Option<RuntimeValue> {
        let ptr = self.read(ptr).unwrap();

        match ptr {
            RuntimeValue::Pointer {
                storage: Storage::Function,
                id,
            } => Some(
                self.function_memory
                    .last()
                    .unwrap()
                    .get(&id)
                    .unwrap()
                    .clone(),
            ),
            _ => None,
        }
    }
}
