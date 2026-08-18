use crate::id_types::{BlockId, FunctionId, TypeId, ValueId};
use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
};

#[derive(Debug, Clone)]
pub enum Type {
    Void,
    Bool,
    Int { bits: u32, signed: bool },
}

#[derive(Debug, Clone)]
pub struct RuntimeValue {
    pub value_type: Type,
    pub value: Vec<u8>,
}

impl RuntimeValue {
    pub fn pretty(&self) -> String {
        match self.value_type {
            Type::Void => String::from("Void"),
            Type::Bool => {
                if self.value[0] == 0 {
                    String::from("False")
                } else {
                    String::from("True")
                }
            }

            Type::Int { bits, signed } => {
                if bits == 32 && signed {
                    let v = i32::from_ne_bytes(self.value.clone().try_into().unwrap());
                    format!("i32({v})").to_string()
                } else if bits == 32 && !signed {
                    let v = u32::from_ne_bytes(self.value.clone().try_into().unwrap());
                    format!("u32({v})").to_string()
                } else {
                    String::from("todo")
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Unknown,
    IAdd(ValueId, TypeId, ValueId, ValueId),
    Call(Option<ValueId>, FunctionId, Vec<ValueId>),
}

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

    pub typemap: HashMap<TypeId, Type>,
    pub blocks: HashMap<BlockId, Block>,
    pub values: HashMap<ValueId, RuntimeValue>,
}

impl Program {
    pub fn vals(&self) {
        for (v_id, v) in self.values.iter() {
            println!("%{v_id}: {}", v.pretty());
        }
    }
}
