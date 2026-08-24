use crate::id_types::{BlockId, FunctionId, MemValueId, TypeId, ValueId};
use crate::program::Program;
use miette::Report;
use std::fmt::{Debug, format};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    Vec { lenght: usize, inner: TypeId },
    Struct { members: Vec<TypeId> },
}

impl From<RuntimeScalarValue> for RuntimeValue {
    fn from(value: RuntimeScalarValue) -> Self {
        RuntimeValue::Scalar(value)
    }
}

impl TryFrom<RuntimeValue> for RuntimeScalarValue {
    type Error = Report;

    fn try_from(value: RuntimeValue) -> Result<Self, Self::Error> {
        match value {
            RuntimeValue::Scalar(v) => Ok(v),
            _ => Err(Report::msg(format!(
                "Failed to get scalar from {:?}",
                value
            ))),
        }
    }
}

#[macro_export]
macro_rules! from_type {
    ( $t1:ident, $t2:ident ) => {
        impl From<$t1> for RuntimeValue {
            fn from(value: $t1) -> Self {
                RuntimeValue::Scalar(RuntimeScalarValue::$t2(value))
            }
        }
        impl From<$t1> for RuntimeScalarValue {
            fn from(value: $t1) -> Self {
                RuntimeScalarValue::$t2(value)
            }
        }
    };
}

from_type!(bool, Bool);
from_type!(u8, U8);
from_type!(u16, U16);
from_type!(u32, U32);
from_type!(u64, U64);
from_type!(i8, I8);
from_type!(i16, I16);
from_type!(i32, I32);
from_type!(i64, I64);

impl TryInto<usize> for RuntimeValue {
    type Error = Report;

    fn try_into(self) -> Result<usize, Self::Error> {
        match self {
            RuntimeValue::Scalar(RuntimeScalarValue::U32(v)) => Ok(v as usize),
            RuntimeValue::Scalar(RuntimeScalarValue::I32(v)) => Ok(v as usize),
            _ => Err(Report::msg(format!("Failed to turn {:?} into u32", self))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeScalarValue {
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pointer {
    pub storage_id: u32,
    pub id: MemValueId,
    pub offsets: Vec<usize>,
}

impl From<Pointer> for RuntimeValue {
    fn from(value: Pointer) -> Self {
        RuntimeValue::Pointer(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeValue {
    Null,
    Void,
    Scalar(RuntimeScalarValue),
    Pointer(Pointer),
    Vec {
        lenght: usize,
        contents: Vec<RuntimeScalarValue>,
    },
    Struct {
        members: Vec<RuntimeValue>,
    },
}

#[macro_export]
macro_rules! vec_from_type {
    ( $t1:ident ) => {
        impl From<Vec<$t1>> for RuntimeValue {
            fn from(value: Vec<$t1>) -> Self {
                RuntimeValue::Vec {
                    lenght: value.len(),
                    contents: value.iter().map(|v| (*v).into()).collect(),
                }
            }
        }
    };
}

vec_from_type!(i64);
vec_from_type!(i32);
vec_from_type!(i16);
vec_from_type!(i8);
vec_from_type!(u64);
vec_from_type!(u32);
vec_from_type!(u16);
vec_from_type!(u8);

impl RuntimeValue {
    pub fn pretty(&self) -> String {
        match self {
            RuntimeValue::Null => String::from("Null"),
            RuntimeValue::Void => String::from("Void"),
            RuntimeValue::Scalar(RuntimeScalarValue::Bool(b)) => {
                if *b {
                    String::from("False")
                } else {
                    String::from("True")
                }
            }

            RuntimeValue::Scalar(RuntimeScalarValue::I8(v)) => format!("i8({v})").to_string(),
            RuntimeValue::Scalar(RuntimeScalarValue::U8(v)) => format!("u8({v})").to_string(),
            RuntimeValue::Scalar(RuntimeScalarValue::I16(v)) => format!("i16({v})").to_string(),
            RuntimeValue::Scalar(RuntimeScalarValue::U16(v)) => format!("u16({v})").to_string(),
            RuntimeValue::Scalar(RuntimeScalarValue::I32(v)) => format!("i32({v})").to_string(),
            RuntimeValue::Scalar(RuntimeScalarValue::U32(v)) => format!("u32({v})").to_string(),
            RuntimeValue::Scalar(RuntimeScalarValue::I64(v)) => format!("i64({v})").to_string(),
            RuntimeValue::Scalar(RuntimeScalarValue::U64(v)) => format!("u64({v})").to_string(),

            RuntimeValue::Pointer(Pointer {
                storage_id,
                id,
                offsets,
            }) => format!("Pointer[{:?}] to {:?}", storage_id, id).to_string(),

            RuntimeValue::Struct { members } => format!(
                "Struct [{}]",
                members
                    .iter()
                    .map(|v| { v.pretty() })
                    .collect::<Vec<String>>()
                    .join(",")
            )
            .to_string(),
            RuntimeValue::Vec { lenght, contents } => format!(
                "Vec{lenght} [{}]",
                contents
                    .iter()
                    .map(|v| {
                        let v: RuntimeValue = v.clone().into();
                        v.pretty()
                    })
                    .collect::<Vec<String>>()
                    .join(",")
            )
            .to_string(),
        }
    }
}
