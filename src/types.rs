use crate::id_types::{MemValueId, TypeId};
use miette::Report;
use num::traits::Euclid;
use std::{
    fmt::Debug,
    ops::{Div, Mul},
};

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

impl RuntimeScalarValue {
    pub fn add(&self, rhs: &RuntimeScalarValue) -> RuntimeScalarValue {
        match (self, rhs) {
            (Self::I8(v1), Self::I8(v2)) => v1.wrapping_add(*v2).into(),
            (Self::I16(v1), Self::I16(v2)) => v1.wrapping_add(*v2).into(),
            (Self::I32(v1), Self::I32(v2)) => v1.wrapping_add(*v2).into(),
            (Self::I64(v1), Self::I64(v2)) => v1.wrapping_add(*v2).into(),
            (Self::U8(v1), Self::U8(v2)) => v1.wrapping_add(*v2).into(),
            (Self::U16(v1), Self::U16(v2)) => v1.wrapping_add(*v2).into(),
            (Self::U32(v1), Self::U32(v2)) => v1.wrapping_add(*v2).into(),
            (Self::U64(v1), Self::U64(v2)) => v1.wrapping_add(*v2).into(),
            (lhs, rhs) => panic!("Mismatched types for add {:?}, {:?}", lhs, rhs),
        }
    }

    pub fn sub(&self, rhs: &RuntimeScalarValue) -> RuntimeScalarValue {
        match (self, rhs) {
            (Self::I8(v1), Self::I8(v2)) => v1.wrapping_sub(*v2).into(),
            (Self::I16(v1), Self::I16(v2)) => v1.wrapping_sub(*v2).into(),
            (Self::I32(v1), Self::I32(v2)) => v1.wrapping_sub(*v2).into(),
            (Self::I64(v1), Self::I64(v2)) => v1.wrapping_sub(*v2).into(),
            (Self::U8(v1), Self::U8(v2)) => v1.wrapping_sub(*v2).into(),
            (Self::U16(v1), Self::U16(v2)) => v1.wrapping_sub(*v2).into(),
            (Self::U32(v1), Self::U32(v2)) => v1.wrapping_sub(*v2).into(),
            (Self::U64(v1), Self::U64(v2)) => v1.wrapping_sub(*v2).into(),
            (lhs, rhs) => panic!("Mismatched types for sub {:?}, {:?}", lhs, rhs),
        }
    }

    pub fn mul(&self, rhs: &RuntimeScalarValue) -> RuntimeScalarValue {
        match (self, rhs) {
            (Self::I8(v1), Self::I8(v2)) => v1.mul(*v2).into(),
            (Self::I16(v1), Self::I16(v2)) => v1.mul(*v2).into(),
            (Self::I32(v1), Self::I32(v2)) => v1.mul(*v2).into(),
            (Self::I64(v1), Self::I64(v2)) => v1.mul(*v2).into(),
            (Self::U8(v1), Self::U8(v2)) => v1.mul(*v2).into(),
            (Self::U16(v1), Self::U16(v2)) => v1.mul(*v2).into(),
            (Self::U32(v1), Self::U32(v2)) => v1.mul(*v2).into(),
            (Self::U64(v1), Self::U64(v2)) => v1.mul(*v2).into(),
            (lhs, rhs) => panic!("Mismatched types for mul {:?}, {:?}", lhs, rhs),
        }
    }

    pub fn div(&self, rhs: &RuntimeScalarValue) -> RuntimeScalarValue {
        match (self, rhs) {
            (Self::I8(v1), Self::I8(v2)) => v1.div(*v2).into(),
            (Self::I16(v1), Self::I16(v2)) => v1.div(*v2).into(),
            (Self::I32(v1), Self::I32(v2)) => v1.div(*v2).into(),
            (Self::I64(v1), Self::I64(v2)) => v1.div(*v2).into(),
            (Self::U8(v1), Self::U8(v2)) => v1.div(*v2).into(),
            (Self::U16(v1), Self::U16(v2)) => v1.div(*v2).into(),
            (Self::U32(v1), Self::U32(v2)) => v1.div(*v2).into(),
            (Self::U64(v1), Self::U64(v2)) => v1.div(*v2).into(),
            (lhs, rhs) => panic!("Mismatched types for div {:?}, {:?}", lhs, rhs),
        }
    }

    pub fn rem(&self, rhs: &RuntimeScalarValue) -> RuntimeScalarValue {
        match (self, rhs) {
            (Self::I8(v1), Self::I8(v2)) => (v1 % v2).into(),
            (Self::I16(v1), Self::I16(v2)) => (v1 % v2).into(),
            (Self::I32(v1), Self::I32(v2)) => (v1 % v2).into(),
            (Self::I64(v1), Self::I64(v2)) => (v1 % v2).into(),
            (Self::U8(v1), Self::U8(v2)) => (v1 % v2).into(),
            (Self::U16(v1), Self::U16(v2)) => (v1 % v2).into(),
            (Self::U32(v1), Self::U32(v2)) => (v1 % v2).into(),
            (Self::U64(v1), Self::U64(v2)) => (v1 % v2).into(),
            (lhs, rhs) => panic!("Mismatched types for div {:?}, {:?}", lhs, rhs),
        }
    }

    pub fn modulus(&self, rhs: &RuntimeScalarValue) -> RuntimeScalarValue {
        match (self, rhs) {
            (Self::I8(v1), Self::I8(v2)) => v1.rem_euclid(v2).into(),
            (Self::I16(v1), Self::I16(v2)) => v1.rem_euclid(v2).into(),
            (Self::I32(v1), Self::I32(v2)) => v1.rem_euclid(v2).into(),
            (Self::I64(v1), Self::I64(v2)) => v1.rem_euclid(v2).into(),
            (Self::U8(v1), Self::U8(v2)) => v1.rem_euclid(v2).into(),
            (Self::U16(v1), Self::U16(v2)) => v1.rem_euclid(v2).into(),
            (Self::U32(v1), Self::U32(v2)) => v1.rem_euclid(v2).into(),
            (Self::U64(v1), Self::U64(v2)) => v1.rem_euclid(v2).into(),
            (lhs, rhs) => panic!("Mismatched types for div {:?}, {:?}", lhs, rhs),
        }
    }

    pub fn and(&self, rhs: &RuntimeScalarValue) -> RuntimeScalarValue {
        match (self, rhs) {
            (Self::Bool(v1), Self::Bool(v2)) => (*v1 && *v2).into(),
            (lhs, rhs) => panic!("Mismatched types for and {:?}, {:?}", lhs, rhs),
        }
    }
    pub fn or(&self, rhs: &RuntimeScalarValue) -> RuntimeScalarValue {
        match (self, rhs) {
            (Self::Bool(v1), Self::Bool(v2)) => (*v1 || *v2).into(),
            (lhs, rhs) => panic!("Mismatched types for or {:?}, {:?}", lhs, rhs),
        }
    }
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
vec_from_type!(bool);

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
                offsets: _,
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

    pub fn modify_inner(&self, mut offsets: Vec<usize>, value: RuntimeValue) -> RuntimeValue {
        if offsets.is_empty() {
            return value;
        }
        let offset = offsets.remove(0);
        match self {
            RuntimeValue::Vec { lenght, contents } => {
                let mut contents = contents.clone();
                let val: RuntimeValue = contents[offset].clone().into();
                contents[offset] = val.modify_inner(offsets, value).try_into().unwrap();
                RuntimeValue::Vec {
                    lenght: *lenght,
                    contents,
                }
            }

            RuntimeValue::Struct { members } => {
                let mut members = members.clone();
                let val = members[offset].clone();
                members[offset] = val.modify_inner(offsets, value);
                RuntimeValue::Struct { members }
            }
            _ => panic!(),
        }
    }

    pub fn read_inner(&self, mut offsets: Vec<usize>) -> RuntimeValue {
        if offsets.is_empty() {
            return self.clone();
        }
        let offset = offsets.remove(0);
        match self {
            RuntimeValue::Vec {
                lenght: _,
                contents,
            } => Into::<RuntimeValue>::into(contents[offset].clone()).read_inner(offsets),
            RuntimeValue::Struct { members } => members[offset].clone().read_inner(offsets),
            _ => panic!(),
        }
    }

    pub fn map_scalars(
        &self,
        rhs: &RuntimeValue,
        f: impl Fn(&RuntimeScalarValue, &RuntimeScalarValue) -> RuntimeScalarValue,
    ) -> RuntimeValue {
        match (self, rhs) {
            (
                Self::Vec {
                    lenght: _,
                    contents: op1,
                },
                Self::Vec {
                    lenght: _,
                    contents: op2,
                },
            ) => {
                let contents = op1
                    .iter()
                    .zip(op2.iter())
                    .map(|(v1, v2)| f(v1, v2))
                    .collect::<Vec<RuntimeScalarValue>>();
                RuntimeValue::Vec {
                    lenght: contents.len(),
                    contents,
                }
            }
            (Self::Scalar(op1), Self::Scalar(op2)) => f(op1, op2).into(),
            (lhs, rhs) => panic!("Cannot map scalar for types: {:?}, {:?}", lhs, rhs),
        }
    }
}
