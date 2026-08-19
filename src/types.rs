use crate::id_types::{BlockId, FunctionId, TypeId, ValueId};
use crate::program::Program;
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
