use crate::types::Storage;

use crate::id_types::{BlockId, FunctionId, ValueId};

#[derive(Debug, Clone)]
pub enum Instruction {
    Alloc {
        out: ValueId,
        storage: Storage,
        init: Option<ValueId>,
    },
    CreateStruct {
        out: ValueId,
        members: Vec<ValueId>,
    },
    CreateVec {
        out: ValueId,
        members: Vec<ValueId>,
    },
    Add(ValueId, ValueId, ValueId),
    Sub(ValueId, ValueId, ValueId),
    Equal(ValueId, ValueId, ValueId),
    LessThan(ValueId, ValueId, ValueId),
    GreaterThan(ValueId, ValueId, ValueId),
    LessThanEq(ValueId, ValueId, ValueId),
    GreaterThanEq(ValueId, ValueId, ValueId),
    VecAllTrue(ValueId, ValueId),
    Select {
        out: ValueId,
        condition: ValueId,
        op1: ValueId,
        op2: ValueId,
    },
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
    CopyInner {
        out: ValueId,
        composite: ValueId,
        offsets: Vec<usize>,
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
