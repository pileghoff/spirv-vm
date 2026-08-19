use crate::{
    id_types::{FunctionId, TypeId, ValueId},
    types::Storage,
};

#[derive(Debug, Clone)]
pub enum Instruction {
    Alloc {
        out: ValueId,
        storage: Storage,
        init: Option<ValueId>,
    },
    IAdd(ValueId, ValueId, ValueId),
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
