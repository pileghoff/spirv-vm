use std::collections::HashMap;

use crate::{
    id_types::{BlockId, FunctionId, TypeId, ValueId},
    instructions::{Instruction, Terminator},
    types::{RuntimeValue, Type},
};

#[derive(Debug, Clone)]
pub struct Block {
    pub instructions: Vec<Instruction>,
    pub terminator: Terminator,
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

    pub typemap: HashMap<TypeId, Type>,
    pub blocks: HashMap<BlockId, Block>,
    pub values: HashMap<ValueId, RuntimeValue>,
    pub values_name: HashMap<ValueId, String>,

    pub source: Option<String>,
}

impl Program {
    pub fn vals(&self) {
        for (v_id, v) in self.values.iter() {
            if let Some(name) = self.values_name.get(v_id) {
                println!("%{name}: {}", v.pretty());
            } else {
                println!("%{v_id}: {}", v.pretty());
            }
        }
    }

    pub fn read(&self, id: &ValueId) -> Option<RuntimeValue> {
        self.values.get(id).cloned()
    }

    pub fn write(&mut self, id: &ValueId, value: RuntimeValue) {
        self.values.insert(*id, value);
    }
}
