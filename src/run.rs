use crate::execution_context::ExecutionNext;
use crate::instructions::{Instruction, Terminator};

use crate::program::Program;

use crate::{execution_context::ExecutionContex, types::*};

pub fn run(program: Program) -> ExecutionContex {
    let mut context = ExecutionContex::new(program);

    while !context.stopped() {
        context.step();
    }
    context
}
