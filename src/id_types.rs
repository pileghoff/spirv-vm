use std::{error::Error, fmt::Display};

#[derive(Debug, Clone)]
pub struct TypeConversionError {
    from: rspirv::dr::Operand,
}

impl Display for TypeConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to convert {:?}", self.from)
    }
}

impl Error for TypeConversionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

pub type GenericId = usize;

#[macro_export]
macro_rules! id_type {
    ( $id:ident ) => {
        #[derive(Debug, Clone, Copy, Default, PartialEq, Hash, Eq)]
        pub struct $id(GenericId);

        impl Display for $id {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<u32> for $id {
            fn from(value: u32) -> Self {
                Self(value as GenericId)
            }
        }

        impl From<usize> for $id {
            fn from(value: usize) -> Self {
                Self(value as GenericId)
            }
        }

        impl From<$id> for usize {
            fn from(value: $id) -> usize {
                value.0
            }
        }

        impl TryFrom<&rspirv::dr::Operand> for $id {
            type Error = TypeConversionError;

            fn try_from(value: &rspirv::dr::Operand) -> Result<Self, Self::Error> {
                match value {
                    rspirv::dr::Operand::LiteralBit32(i) => Ok(Self(*i as GenericId)),
                    rspirv::dr::Operand::IdRef(i) => Ok(Self(*i as GenericId)),
                    _ => Err(Self::Error {
                        from: value.clone(),
                    }),
                }
            }
        }
    };
}

id_type!(FunctionId);
id_type!(TypeId);
id_type!(BlockId);
id_type!(ValueId);
id_type!(MemValueId);
