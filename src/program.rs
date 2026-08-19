use std::collections::HashMap;

use crate::{
    id_types::{BlockId, FunctionId, TypeId, ValueId},
    instructions::Instruction,
    types::{RuntimeValue, Type},
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

    pub function_stack: Vec<(BlockId, usize, Option<ValueId>)>,
    pub function_memory: Vec<HashMap<ValueId, RuntimeValue>>,
    pub typemap: HashMap<TypeId, Type>,
    pub blocks: HashMap<BlockId, Block>,
    pub values: HashMap<ValueId, RuntimeValue>,
    pub values_name: HashMap<ValueId, String>,
}

impl Program {
    pub fn vals(&self) {
        for (v_id, v) in self.values.iter() {
            if let Some(name) = self.values_name.get(v_id) {
                println!("%{name}: {}", v.pretty(self));
            } else {
                println!("%{v_id}: {}", v.pretty(self));
            }
        }
    }
}
