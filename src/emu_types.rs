use crate::id_types::{BlockId, FunctionId, TypeId, ValueId};
use std::{collections::HashMap, fmt::Debug};

#[derive(Debug, Clone)]
pub enum Storage {
    Function,
}

impl From<rspirv::dr::Operand> for Storage {
    fn from(value: rspirv::dr::Operand) -> Self {
        match value {
            rspirv::dr::Operand::StorageClass(rspirv::spirv::StorageClass::Function) => {
                Storage::Function
            }
            _ => panic!("Ups {:?}", value),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    Null,
    Void,
    Bool,
    Int { bits: u32, signed: bool },
    Pointer { storage: Storage, inner: TypeId },
}

#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Null,
    Void,
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    Pointer { storage: Storage, id: ValueId },
}

impl RuntimeValue {
    pub fn pretty(&self, program: &Program) -> String {
        match self {
            RuntimeValue::Null => String::from("Null"),
            RuntimeValue::Void => String::from("Void"),
            RuntimeValue::Bool(b) => {
                if *b {
                    String::from("False")
                } else {
                    String::from("True")
                }
            }

            RuntimeValue::I8(v) => format!("i8({v})").to_string(),
            RuntimeValue::U8(v) => format!("u8({v})").to_string(),
            RuntimeValue::I16(v) => format!("i16({v})").to_string(),
            RuntimeValue::U16(v) => format!("u16({v})").to_string(),
            RuntimeValue::I32(v) => format!("i32({v})").to_string(),
            RuntimeValue::U32(v) => format!("u32({v})").to_string(),
            RuntimeValue::I64(v) => format!("i64({v})").to_string(),
            RuntimeValue::U64(v) => format!("u64({v})").to_string(),

            RuntimeValue::Pointer {
                storage: Storage::Function,
                id,
            } => {
                let value = program.function_memory.last().unwrap().get(id).unwrap();
                format!("Pointer to {:?} -> {}", id, value.pretty(program)).to_string()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Alloc {
        out: ValueId,
        storage: Storage,
        t_id: TypeId,
        init: Option<ValueId>,
    },
    IAdd(ValueId, TypeId, ValueId, ValueId),
    IEqual(ValueId, ValueId, ValueId),
    Call(Option<ValueId>, FunctionId, Vec<ValueId>),
    Load {
        out: ValueId,
        ptr: ValueId,
    },
    Store {
        from: ValueId,
        ptr: ValueId,
    },
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
