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

fn op_to_string(op: &rspirv::dr::Operand) -> Option<String> {
    match op {
        rspirv::dr::Operand::LiteralString(i) => Some(String::from(i)),
        _ => None,
    }
}

fn parse_block(
    insts: &mut VecDeque<rspirv::dr::Instruction>,
    valuemap: &HashMap<ValueId, RuntimeValue>,
    typemap: &HashMap<TypeId, Type>,
) -> Block {
    let mut instructions = Vec::new();
    let terminator: Terminator = loop {
        let inst = insts.pop_front().unwrap();
        match inst.class.opcode {
            rspirv::spirv::Op::Branch => {
                let i: BlockId = (&inst.operands[0]).try_into().unwrap();
                break Terminator::Jump(i);
            }

            rspirv::spirv::Op::BranchConditional => {
                let condition: ValueId = (&inst.operands[0]).try_into().unwrap();
                let then_block: BlockId = (&inst.operands[1]).try_into().unwrap();
                let else_block: BlockId = (&inst.operands[2]).try_into().unwrap();
                break Terminator::Branch {
                    condition,
                    then_block,
                    else_block,
                };
            }
            rspirv::spirv::Op::SelectionMerge => {
                println!("We ignore selectionMerge");
            }
            rspirv::spirv::Op::Return => break Terminator::Return(None),
            rspirv::spirv::Op::ReturnValue => {
                let v_id = (&inst.operands[0]).try_into().unwrap();
                break Terminator::Return(Some(v_id));
            }
            rspirv::spirv::Op::Variable => {
                let v_id: ValueId = inst.result_id.unwrap().into();
                let t_id: TypeId = inst.result_type.unwrap().into();
                let storage: Storage = inst.operands[0].clone().into();
                let init: Option<ValueId> = if let Some(v) = inst.operands.get(1) {
                    Some((v).try_into().unwrap())
                } else {
                    None
                };

                instructions.push(Instruction::Alloc {
                    out: v_id,
                    storage,
                    t_id,
                    init,
                });
            }
            rspirv::spirv::Op::Load => {
                let v_id: ValueId = inst.result_id.unwrap().into();
                let op: ValueId = (&inst.operands[0]).try_into().unwrap();
                instructions.push(Instruction::Load { out: v_id, ptr: op });
            }
            rspirv::spirv::Op::Store => {
                let ptr: ValueId = (&inst.operands[0]).try_into().unwrap();
                let from: ValueId = (&inst.operands[1]).try_into().unwrap();
                instructions.push(Instruction::Store { from, ptr });
            }
            rspirv::spirv::Op::IAdd => {
                let v_id = inst.result_id.unwrap().into();
                let t_id = inst.result_type.unwrap().into();
                let op1 = (&inst.operands[0]).try_into().unwrap();
                let op2 = (&inst.operands[1]).try_into().unwrap();
                instructions.push(Instruction::IAdd(v_id, t_id, op1, op2));
            }
            rspirv::spirv::Op::IEqual => {
                let v_id = inst.result_id.unwrap().into();
                let op1 = (&inst.operands[0]).try_into().unwrap();
                let op2 = (&inst.operands[1]).try_into().unwrap();
                instructions.push(Instruction::IEqual(v_id, op1, op2));
            }
            rspirv::spirv::Op::FunctionCall => {
                let r_id = inst.result_id.unwrap().into();
                let mut ops = inst.operands.clone();
                let f_id: FunctionId = (&ops.remove(0)).try_into().unwrap();
                let args = ops.iter().map(|op| op.try_into().unwrap()).collect();
                instructions.push(Instruction::Call(Some(r_id), f_id, args));
            }
            rspirv::spirv::Op::Line => println!("Ignore line"),
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
    valuemap: &HashMap<ValueId, RuntimeValue>,
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
                let b = parse_block(insts, valuemap, typemap);
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
            rspirv::spirv::Op::Name => {
                let v_id = (&inst.operands[0]).try_into().unwrap();
                let name = op_to_string(&inst.operands[1]).unwrap();
                program.values_name.insert(v_id, name);
            }
            rspirv::spirv::Op::EntryPoint => {
                program.entry_point = (&inst.operands[1]).try_into().unwrap();
            }
            rspirv::spirv::Op::ConstantNull => {
                let v_id = inst.result_id.unwrap().into();
                program.values.insert(v_id, RuntimeValue::Null);
            }
            rspirv::spirv::Op::TypeVoid => {
                let i = inst.result_id.unwrap().into();
                println!("Save type Void into %{}", i);
                program.typemap.insert(i, Type::Void);
            }
            rspirv::spirv::Op::TypeBool => {
                let i = inst.result_id.unwrap().into();
                println!("Save type Bool into %{}", i);
                program.typemap.insert(i, Type::Bool);
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
                    } => RuntimeValue::U32(op_to_u32(&inst.operands[0]).unwrap()),
                    Type::Int {
                        bits: 32,
                        signed: true,
                    } => RuntimeValue::I32(op_to_i32(&inst.operands[0]).unwrap()),
                    _ => panic!("Failed to handle type: {:?}", t),
                };
                println!("Constant %{i}: {:?}", v.pretty(&program));
                program.values.insert(i, v);
            }
            rspirv::spirv::Op::TypePointer => {
                let t_id: TypeId = inst.result_id.unwrap().into();
                let storage: Storage = (inst.operands[0].clone()).into();
                let pt_id: TypeId = (&inst.operands[1]).try_into().unwrap();
                program.typemap.insert(
                    t_id,
                    Type::Pointer {
                        storage,
                        inner: pt_id,
                    },
                );
            }
            rspirv::spirv::Op::Function => {
                let f_id: FunctionId = inst.result_id.unwrap().into();
                program.functions.insert(
                    f_id,
                    parse_func(
                        &mut insts,
                        &program.values,
                        &program.typemap,
                        &mut program.blocks,
                    ),
                );
            }
            rspirv::spirv::Op::Capability
            | rspirv::spirv::Op::ExtInstImport
            | rspirv::spirv::Op::MemoryModel
            | rspirv::spirv::Op::Line
            | rspirv::spirv::Op::String
            | rspirv::spirv::Op::Source
            | rspirv::spirv::Op::ExecutionMode => println!("Ignored {:?}", inst.class.opcode),
            _ => println!("Unknown opcode {:?}", inst),
        }
    }

    program
}
