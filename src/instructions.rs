use crate::types::Storage;

use crate::id_types::{BlockId, FunctionId, ValueId};

#[derive(Debug, Clone)]
pub enum Instruction {
    Alloc {
        out: ValueId,
        storage: Storage,
        init: Option<ValueId>,
    },
    IAdd(ValueId, ValueId, ValueId),
    IEqual(ValueId, ValueId, ValueId),
    ILessThan(ValueId, ValueId, ValueId),
    IGreaterThan(ValueId, ValueId, ValueId),
    ILessThanEq(ValueId, ValueId, ValueId),
    IGreaterThanEq(ValueId, ValueId, ValueId),
    Call(Option<ValueId>, FunctionId, Vec<ValueId>),
    Load {
        out: ValueId,
        ptr: ValueId,
    },
    Store {
        from: ValueId,
        ptr: ValueId,
    },
    CreateInnerPointer {
        out: ValueId,
        base: ValueId,
        offsets: Vec<ValueId>,
    },
    Line(u32),
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
