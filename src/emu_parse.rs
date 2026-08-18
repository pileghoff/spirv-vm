use crate::emu_types::*;
use crate::id_types::{BlockId, FunctionId, TypeId, ValueId};
use std::collections::{HashMap, VecDeque};
use std::io::Read;
use std::{fs::File, io::BufReader};

fn op_to_u32(op: &rspirv::dr::Operand) -> Option<u32> {
    match op {
        rspirv::dr::Operand::LiteralBit32(i) => Some(*i),
        rspirv::dr::Operand::IdRef(i) => Some(*i),
        _ => None,
    }
}

fn op_to_i32(op: &rspirv::dr::Operand) -> Option<i32> {
    match op {
        rspirv::dr::Operand::LiteralBit32(i) => Some(*i as i32),
        _ => None,
    }
}

fn parse_block(insts: &mut VecDeque<rspirv::dr::Instruction>) -> Block {
    let mut instructions = Vec::new();
    let terminator: Terminator = loop {
        let inst = insts.pop_front().unwrap();
        match inst.class.opcode {
            rspirv::spirv::Op::Branch => {
                let i: BlockId = (&inst.operands[0]).try_into().unwrap();
                break Terminator::Jump(i);
            }
            rspirv::spirv::Op::Return => break Terminator::Return(None),
            rspirv::spirv::Op::ReturnValue => {
                let v_id = (&inst.operands[0]).try_into().unwrap();
                break Terminator::Return(Some(v_id));
            }
            rspirv::spirv::Op::IAdd => {
                let v_id = inst.result_id.unwrap().into();
                let t_id = inst.result_type.unwrap().into();
                let op1 = (&inst.operands[0]).try_into().unwrap();
                let op2 = (&inst.operands[1]).try_into().unwrap();
                instructions.push(Instruction::IAdd(v_id, t_id, op1, op2));
            }
            rspirv::spirv::Op::FunctionCall => {
                let r_id = inst.result_id.unwrap().into();
                let mut ops = inst.operands.clone();
                let f_id: FunctionId = (&ops.remove(0)).try_into().unwrap();
                let args = ops.iter().map(|op| op.try_into().unwrap()).collect();
                instructions.push(Instruction::Call(Some(r_id), f_id, args));
            }
            _ => {
                println!("unknown inst {:?}", inst)
            }
        }
    };

    Block {
        instructions,
        terminator,
    }
}

fn parse_func(
    insts: &mut VecDeque<rspirv::dr::Instruction>,
    typemap: &HashMap<TypeId, Type>,
    blocks: &mut HashMap<BlockId, Block>,
) -> Function {
    let mut func = Function {
        blocks: Vec::new(),
        args: Vec::new(),
    };

    while !insts.is_empty() {
        let inst = insts.pop_front().unwrap();
        match inst.class.opcode {
            rspirv::spirv::Op::FunctionEnd => break,
            rspirv::spirv::Op::Label => {
                println!("Start block");
                let i = inst.result_id.unwrap().into();
                let b = parse_block(insts);
                println!("Save block in %{i}: {:?}", b);
                blocks.insert(i, b);
                func.blocks.push(i);
            }
            rspirv::spirv::Op::FunctionParameter => {
                let t_id = inst.result_type.unwrap().into();
                let v_id = inst.result_id.unwrap().into();
                let t = typemap.get(&t_id).unwrap().clone();
                func.args.push(v_id);
            }
            _ => println!("Unknown inst while parsing function: {:?}", inst),
        }
    }

    func
}

pub fn parse(path: &str) -> Program {
    let buf = BufReader::new(File::open(path).unwrap());
    let bytes: Vec<u8> = buf.bytes().map(|b| b.unwrap()).collect();
    let module = rspirv::dr::load_bytes(bytes).unwrap();

    let mut program = Program::default();
    let mut insts: VecDeque<rspirv::dr::Instruction> = module.all_inst_iter().cloned().collect();

    while !insts.is_empty() {
        let inst = insts.pop_front().unwrap();
        match inst.class.opcode {
            rspirv::spirv::Op::EntryPoint => {
                program.entry_point = (&inst.operands[1]).try_into().unwrap();
            }
            rspirv::spirv::Op::TypeVoid => {
                let i = inst.result_id.unwrap().into();
                println!("Save type Void into %{}", i);
                program.typemap.insert(i, Type::Void);
            }
            rspirv::spirv::Op::TypeInt => {
                let i = inst.result_id.unwrap().into();
                let size = op_to_u32(&inst.operands[0]).unwrap();
                let signed = op_to_u32(&inst.operands[1]).unwrap() == 1;
                println!(
                    "Save type {}{} into %{}",
                    if signed { "i" } else { "u" },
                    size,
                    i
                );

                program.typemap.insert(i, Type::Int { bits: size, signed });
            }
            rspirv::spirv::Op::TypeFunction => {
                println!("We dont care aobut function types");
            }
            rspirv::spirv::Op::Constant => {
                let i = inst.result_id.unwrap().into();
                let tid = inst.result_type.unwrap().into();
                let t = program.typemap.get(&tid).unwrap();

                let v = match t {
                    Type::Int {
                        bits: 32,
                        signed: false,
                    } => Vec::from(op_to_u32(&inst.operands[0]).unwrap().to_ne_bytes()),
                    Type::Int {
                        bits: 32,
                        signed: true,
                    } => Vec::from(op_to_i32(&inst.operands[0]).unwrap().to_ne_bytes()),
                    _ => panic!("Failed to handle type: {:?}", t),
                };
                println!("Constant %{i}: {:?}", v);
                program.values.insert(
                    i,
                    RuntimeValue {
                        value_type: t.clone(),
                        value: v,
                    },
                );
            }
            rspirv::spirv::Op::Function => {
                let f_id: FunctionId = inst.result_id.unwrap().into();
                program.functions.insert(
                    f_id,
                    parse_func(&mut insts, &program.typemap, &mut program.blocks),
                );
            }
            rspirv::spirv::Op::Capability
            | rspirv::spirv::Op::ExtInstImport
            | rspirv::spirv::Op::MemoryModel
            | rspirv::spirv::Op::ExecutionMode => println!("Ignored {:?}", inst.class.opcode),
            _ => println!("Unknown opcode {:?}", inst),
        }
    }

    program
}
