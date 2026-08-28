use crate::id_types::{BlockId, FunctionId, TypeId, ValueId};
use crate::instructions::{Instruction, Terminator};
use crate::program::{Block, Function, Program};
use crate::types::*;
use std::collections::{HashMap, VecDeque};
use std::io::Read;
use std::{fs::File, io::BufReader};

use miette::{IntoDiagnostic, Result, miette};

fn result_id<I>(inst: &rspirv::dr::Instruction) -> Result<I>
where
    I: From<u32>,
{
    inst.result_id
        .map(|v: u32| Into::<I>::into(v))
        .ok_or(miette!(
            "Failed to get result ID from inst {:?}",
            inst.class.opcode
        ))
}

fn op_to_u32(op: &rspirv::dr::Operand) -> Result<u32> {
    match op {
        rspirv::dr::Operand::LiteralBit32(i) => Ok(*i),
        rspirv::dr::Operand::IdRef(i) => Ok(*i),
        _ => Err(miette!("Attempted to extract u32 from {:?}", op)),
    }
}

fn op_to_i32(op: &rspirv::dr::Operand) -> Result<i32> {
    match op {
        rspirv::dr::Operand::LiteralBit32(i) => Ok(*i as i32),
        _ => Err(miette!("Attempted to extract i32 from {:?}", op)),
    }
}

fn op_to_string(op: &rspirv::dr::Operand) -> Result<String> {
    match op {
        rspirv::dr::Operand::LiteralString(i) => Ok(String::from(i)),
        _ => Err(miette!("Attempted to extract string from {:?}", op)),
    }
}

fn parse_block(
    insts: &mut VecDeque<rspirv::dr::Instruction>,
    _valuemap: &HashMap<ValueId, RuntimeValue>,
    typemap: &HashMap<TypeId, Type>,
) -> Result<Block> {
    let mut instructions = Vec::new();
    let terminator: Terminator = loop {
        let inst = insts.pop_front().ok_or(miette!(
            "Ran out of instructions while parsing block. Missing a terminator."
        ))?;
        match inst.class.opcode {
            rspirv::spirv::Op::Branch => {
                let i: BlockId = (&inst.operands[0]).try_into().into_diagnostic()?;
                break Terminator::Jump(i);
            }

            rspirv::spirv::Op::BranchConditional => {
                let condition: ValueId = (&inst.operands[0]).try_into().into_diagnostic()?;
                let then_block: BlockId = (&inst.operands[1]).try_into().into_diagnostic()?;
                let else_block: BlockId = (&inst.operands[2]).try_into().into_diagnostic()?;
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
                let v_id = (&inst.operands[0]).try_into().into_diagnostic()?;
                break Terminator::Return(Some(v_id));
            }
            rspirv::spirv::Op::AccessChain => {
                let out: ValueId = result_id(&inst)?;
                let mut ops = inst.operands.clone();
                let base: ValueId = (&ops.remove(0)).try_into().into_diagnostic()?;
                let offsets = ops
                    .iter()
                    .map(|v| v.try_into().into_diagnostic())
                    .collect::<Result<Vec<_>>>()?;

                instructions.push(Instruction::CreateInnerPointer { out, base, offsets });
            }
            rspirv::spirv::Op::CompositeConstruct => {
                let t_id: TypeId = inst
                    .result_type
                    .map(|v: u32| Into::<TypeId>::into(v))
                    .ok_or(miette!(
                        "Failed to get result ID from inst {:?}",
                        inst.class.opcode
                    ))?;
                let out: ValueId = result_id(&inst)?;
                let members: Vec<ValueId> = inst
                    .operands
                    .iter()
                    .map(|v| TryInto::<ValueId>::try_into(v).into_diagnostic())
                    .collect::<Result<Vec<ValueId>>>()?;
                match typemap.get(&t_id).ok_or(miette!(
                    "Result type not found for CompositeConstruct: %{:?}",
                    t_id
                ))? {
                    Type::Vec {
                        lenght: _,
                        inner: _,
                    } => instructions.push(Instruction::CreateVec { out, members }),
                    Type::Struct { members: _ } => {
                        instructions.push(Instruction::CreateStruct { out, members })
                    }
                    _ => {
                        return Err(miette!(
                            "Incorrect type for CompositeConstruct: %{:?}",
                            t_id
                        ));
                    }
                };
            }
            rspirv::spirv::Op::CompositeExtract => {
                let out: ValueId = result_id(&inst)?;
                let mut ops = inst.operands.clone();
                let composite: ValueId = (&ops.remove(0)).try_into().into_diagnostic()?;
                let offsets = ops
                    .iter()
                    .map(|v| op_to_u32(v).map(|v| v as usize))
                    .collect::<Result<Vec<_>>>()?;

                instructions.push(Instruction::CopyInner {
                    out,
                    composite,
                    offsets,
                });
            }
            rspirv::spirv::Op::Select => {
                let out: ValueId = result_id(&inst)?;
                let condition: ValueId = (&inst.operands[0]).try_into().into_diagnostic()?;
                let op1: ValueId = (&inst.operands[1]).try_into().into_diagnostic()?;
                let op2: ValueId = (&inst.operands[2]).try_into().into_diagnostic()?;
                instructions.push(Instruction::Select {
                    out,
                    condition,
                    op1,
                    op2,
                });
            }
            rspirv::spirv::Op::Variable => {
                let v_id: ValueId = result_id(&inst)?;
                let storage: Storage = inst.operands[0].clone().into();
                let init: Option<ValueId> = inst
                    .operands
                    .get(1)
                    .map(|v| TryInto::<ValueId>::try_into(v).into_diagnostic())
                    .transpose()?;

                instructions.push(Instruction::Alloc {
                    out: v_id,
                    storage,
                    init,
                });
            }
            rspirv::spirv::Op::Load => {
                let v_id: ValueId = result_id(&inst)?;
                let op: ValueId = (&inst.operands[0]).try_into().into_diagnostic()?;
                instructions.push(Instruction::Load { out: v_id, ptr: op });
            }
            rspirv::spirv::Op::Store => {
                let ptr: ValueId = (&inst.operands[0]).try_into().into_diagnostic()?;
                let from: ValueId = (&inst.operands[1]).try_into().into_diagnostic()?;
                instructions.push(Instruction::Store { from, ptr });
            }
            rspirv::spirv::Op::LogicalAnd => {
                let v_id = result_id(&inst)?;
                let op1 = (&inst.operands[0]).try_into().into_diagnostic()?;
                let op2 = (&inst.operands[1]).try_into().into_diagnostic()?;
                instructions.push(Instruction::And(v_id, op1, op2));
            }
            rspirv::spirv::Op::LogicalOr => {
                let v_id = result_id(&inst)?;
                let op1 = (&inst.operands[0]).try_into().into_diagnostic()?;
                let op2 = (&inst.operands[1]).try_into().into_diagnostic()?;
                instructions.push(Instruction::Or(v_id, op1, op2));
            }
            rspirv::spirv::Op::IAdd => {
                let v_id = result_id(&inst)?;
                let op1 = (&inst.operands[0]).try_into().into_diagnostic()?;
                let op2 = (&inst.operands[1]).try_into().into_diagnostic()?;
                instructions.push(Instruction::Add(v_id, op1, op2));
            }
            rspirv::spirv::Op::ISub => {
                let v_id = result_id(&inst)?;
                let op1 = (&inst.operands[0]).try_into().into_diagnostic()?;
                let op2 = (&inst.operands[1]).try_into().into_diagnostic()?;
                instructions.push(Instruction::Sub(v_id, op1, op2));
            }
            rspirv::spirv::Op::UDiv | rspirv::spirv::Op::SDiv => {
                let v_id = result_id(&inst)?;
                let op1 = (&inst.operands[0]).try_into().into_diagnostic()?;
                let op2 = (&inst.operands[1]).try_into().into_diagnostic()?;
                instructions.push(Instruction::Div(v_id, op1, op2));
            }
            rspirv::spirv::Op::IMul => {
                let v_id = result_id(&inst)?;
                let op1 = (&inst.operands[0]).try_into().into_diagnostic()?;
                let op2 = (&inst.operands[1]).try_into().into_diagnostic()?;
                instructions.push(Instruction::Mul(v_id, op1, op2));
            }
            rspirv::spirv::Op::UMod | rspirv::spirv::Op::SMod => {
                let v_id = result_id(&inst)?;
                let op1 = (&inst.operands[0]).try_into().into_diagnostic()?;
                let op2 = (&inst.operands[1]).try_into().into_diagnostic()?;
                instructions.push(Instruction::Mod(v_id, op1, op2));
            }
            rspirv::spirv::Op::SRem => {
                let v_id = result_id(&inst)?;
                let op1 = (&inst.operands[0]).try_into().into_diagnostic()?;
                let op2 = (&inst.operands[1]).try_into().into_diagnostic()?;
                instructions.push(Instruction::Rem(v_id, op1, op2));
            }
            rspirv::spirv::Op::SGreaterThan => {
                let v_id = result_id(&inst)?;
                let op1 = (&inst.operands[0]).try_into().into_diagnostic()?;
                let op2 = (&inst.operands[1]).try_into().into_diagnostic()?;
                instructions.push(Instruction::GreaterThan(v_id, op1, op2));
            }
            rspirv::spirv::Op::SGreaterThanEqual => {
                let v_id = result_id(&inst)?;
                let op1 = (&inst.operands[0]).try_into().into_diagnostic()?;
                let op2 = (&inst.operands[1]).try_into().into_diagnostic()?;
                instructions.push(Instruction::GreaterThanEq(v_id, op1, op2));
            }
            rspirv::spirv::Op::SLessThan => {
                let v_id = result_id(&inst)?;
                let op1 = (&inst.operands[0]).try_into().into_diagnostic()?;
                let op2 = (&inst.operands[1]).try_into().into_diagnostic()?;
                instructions.push(Instruction::LessThan(v_id, op1, op2));
            }
            rspirv::spirv::Op::SLessThanEqual => {
                let v_id = result_id(&inst)?;
                let op1 = (&inst.operands[0]).try_into().into_diagnostic()?;
                let op2 = (&inst.operands[1]).try_into().into_diagnostic()?;
                instructions.push(Instruction::LessThanEq(v_id, op1, op2));
            }
            rspirv::spirv::Op::All => {
                let v_id = result_id(&inst)?;
                let op = (&inst.operands[0]).try_into().into_diagnostic()?;
                instructions.push(Instruction::VecAllTrue(v_id, op));
            }

            rspirv::spirv::Op::IEqual => {
                let v_id = result_id(&inst)?;
                let op1 = (&inst.operands[0]).try_into().into_diagnostic()?;
                let op2 = (&inst.operands[1]).try_into().into_diagnostic()?;
                instructions.push(Instruction::Equal(v_id, op1, op2));
            }
            rspirv::spirv::Op::FunctionCall => {
                let r_id = result_id(&inst)?;
                let mut ops = inst.operands.clone();
                let f_id: FunctionId = (&ops.remove(0)).try_into().into_diagnostic()?;
                let args = ops
                    .iter()
                    .map(|op| op.try_into().into_diagnostic())
                    .collect::<Result<Vec<_>>>()?;
                instructions.push(Instruction::Call(Some(r_id), f_id, args));
            }
            rspirv::spirv::Op::Line => {
                instructions.push(Instruction::Line(op_to_u32(&inst.operands[1])? - 1));
            }
            _ => {
                println!("unknown inst {:?}", inst)
            }
        }
    };

    Ok(Block {
        instructions,
        terminator,
    })
}

fn parse_func(
    insts: &mut VecDeque<rspirv::dr::Instruction>,
    valuemap: &HashMap<ValueId, RuntimeValue>,
    typemap: &HashMap<TypeId, Type>,
    blocks: &mut HashMap<BlockId, Block>,
) -> Result<Function> {
    let mut func = Function {
        blocks: Vec::new(),
        args: Vec::new(),
    };

    while !insts.is_empty() {
        let inst = insts.pop_front().unwrap();
        match inst.class.opcode {
            rspirv::spirv::Op::FunctionEnd => break,
            rspirv::spirv::Op::Label => {
                let i = result_id(&inst)?;
                let b = parse_block(insts, valuemap, typemap)?;
                blocks.insert(i, b);
                func.blocks.push(i);
            }
            rspirv::spirv::Op::FunctionParameter => {
                let t_id = inst.result_type.unwrap().into();
                let v_id = result_id(&inst)?;
                let _t = typemap.get(&t_id).unwrap().clone();
                func.args.push(v_id);
            }
            _ => println!("Unknown inst while parsing function: {:?}", inst),
        }
    }

    Ok(func)
}

pub fn parse(path: &str) -> Result<Program> {
    let buf = BufReader::new(File::open(path).into_diagnostic()?);
    let bytes: Vec<u8> = buf.bytes().map(|b| b.unwrap()).collect();
    parse_bytes(bytes)
}

pub fn parse_bytes(bytes: Vec<u8>) -> Result<Program> {
    let module = rspirv::dr::load_bytes(bytes).unwrap();
    parse_module(module)
}

pub fn parse_words(words: Vec<u32>) -> Result<Program> {
    let module = rspirv::dr::load_words(words).unwrap();
    parse_module(module)
}

fn parse_module(module: rspirv::dr::Module) -> Result<Program> {
    let mut program = Program::default();
    let mut insts: VecDeque<rspirv::dr::Instruction> = module.all_inst_iter().cloned().collect();

    while !insts.is_empty() {
        let inst = insts.pop_front().unwrap();
        match inst.class.opcode {
            rspirv::spirv::Op::Name => {
                let v_id = (&inst.operands[0]).try_into().into_diagnostic()?;
                let name = op_to_string(&inst.operands[1]).unwrap();
                program.values_name.insert(v_id, name);
            }
            rspirv::spirv::Op::EntryPoint => {
                program.entry_point = (&inst.operands[1]).try_into().into_diagnostic()?;
            }
            rspirv::spirv::Op::ConstantNull => {
                let v_id = result_id(&inst)?;
                program.values.insert(v_id, RuntimeValue::Null);
            }
            rspirv::spirv::Op::TypeVoid => {
                let i = result_id(&inst)?;
                program.typemap.insert(i, Type::Void);
            }
            rspirv::spirv::Op::TypeBool => {
                let i = result_id(&inst)?;
                program.typemap.insert(i, Type::Bool);
            }

            rspirv::spirv::Op::TypeInt => {
                let i = result_id(&inst)?;
                let size = op_to_u32(&inst.operands[0]).unwrap();
                let signed = op_to_u32(&inst.operands[1]).unwrap() == 1;

                program.typemap.insert(i, Type::Int { bits: size, signed });
            }
            rspirv::spirv::Op::TypeFunction => {
                println!("We dont care aobut function types");
            }
            rspirv::spirv::Op::TypeStruct => {
                let i = result_id(&inst)?;
                let members: Vec<TypeId> = inst
                    .operands
                    .iter()
                    .map(|op| TryInto::<TypeId>::try_into(op).into_diagnostic())
                    .collect::<Result<Vec<TypeId>>>()?;

                program.typemap.insert(i, Type::Struct { members });
            }

            rspirv::spirv::Op::TypeVector => {
                let i = result_id(&inst)?;
                let inner: TypeId = (&inst.operands[0]).try_into().into_diagnostic()?;
                let lenght = op_to_u32(&inst.operands[1]).unwrap() as usize;

                program.typemap.insert(i, Type::Vec { lenght, inner });
            }
            rspirv::spirv::Op::Constant => {
                let i = result_id(&inst)?;
                let tid = inst.result_type.unwrap().into();
                let t = program.typemap.get(&tid).unwrap();

                let v: RuntimeValue = match t {
                    Type::Int {
                        bits: 32,
                        signed: false,
                    } => RuntimeScalarValue::U32(op_to_u32(&inst.operands[0]).unwrap()),
                    Type::Int {
                        bits: 32,
                        signed: true,
                    } => RuntimeScalarValue::I32(op_to_i32(&inst.operands[0]).unwrap()),
                    _ => panic!("Failed to handle type: {:?}", t),
                }
                .into();
                println!("Constant %{i}: {:?}", v.pretty());
                program.values.insert(i, v);
            }
            rspirv::spirv::Op::ConstantComposite => {
                let i: ValueId = result_id(&inst)?;

                let tid = inst.result_type.unwrap().into();
                let t = program.typemap.get(&tid).unwrap();

                let v: RuntimeValue = match t {
                    Type::Vec { lenght, inner: _ } => {
                        let contents = inst
                            .operands
                            .iter()
                            .map(|v| {
                                TryInto::<ValueId>::try_into(v)
                                    .into_diagnostic()
                                    .and_then(|v| {
                                        program
                                            .read(&v)
                                            .ok_or(miette!("Failed to get value"))
                                            .and_then(|v| v.try_into())
                                    })
                            })
                            .collect::<Result<Vec<RuntimeScalarValue>>>()?;
                        RuntimeValue::Vec {
                            lenght: *lenght,
                            contents,
                        }
                    }
                    Type::Struct { members: _ } => {
                        let members =
                            inst.operands
                                .iter()
                                .map(|v| {
                                    TryInto::<ValueId>::try_into(v).into_diagnostic().and_then(
                                        |v| program.read(&v).ok_or(miette!("Failed to get value")),
                                    )
                                })
                                .collect::<Result<Vec<_>>>()?;
                        RuntimeValue::Struct { members }
                    }
                    _ => panic!("Failed to handle type: {:?}", t),
                };

                println!("Constant %{i}: {:?}", v.pretty());
                program.values.insert(i, v);
            }
            rspirv::spirv::Op::TypePointer => {
                let t_id: TypeId = result_id(&inst)?;
                let storage: Storage = (inst.operands[0].clone()).into();
                let pt_id: TypeId = (&inst.operands[1]).try_into().into_diagnostic()?;
                program.typemap.insert(
                    t_id,
                    Type::Pointer {
                        storage,
                        inner: pt_id,
                    },
                );
            }
            rspirv::spirv::Op::Function => {
                let f_id: FunctionId = result_id(&inst)?;
                program.functions.insert(
                    f_id,
                    parse_func(
                        &mut insts,
                        &program.values,
                        &program.typemap,
                        &mut program.blocks,
                    )?,
                );
            }
            rspirv::spirv::Op::Source => {
                if let Some(source) = inst.operands.get(3) {
                    program.source = Some(op_to_string(source)?);
                }
            }
            rspirv::spirv::Op::Capability
            | rspirv::spirv::Op::ExtInstImport
            | rspirv::spirv::Op::MemoryModel
            | rspirv::spirv::Op::String
            | rspirv::spirv::Op::ExecutionMode => println!("Ignored {:?}", inst.class.opcode),
            _ => println!("Unknown opcode {:?}", inst),
        }
    }

    Ok(program)
}
