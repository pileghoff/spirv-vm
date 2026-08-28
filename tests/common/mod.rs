use naga::{
    back::spv::{self, Options, WriterFlags},
    valid::ShaderStages,
};
use spirvemu::run::run;
use spirvemu::types::*;
use spirvemu::{execution_context::ExecutionContex, parse::parse_words};
use spirvemu::{id_types::*, program::Program};

pub fn compile(source: &str) -> Vec<u32> {
    let module: naga::Module = naga::front::wgsl::parse_str(source).unwrap();
    let module_info: naga::valid::ModuleInfo = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .subgroup_stages(naga::valid::ShaderStages::all())
    .subgroup_operations(naga::valid::SubgroupOperationSet::all())
    .validate(&module)
    .unwrap();
    let options = Options::default();

    spv::write_vec(&module, &module_info, &options, None).unwrap()
}

pub fn run_source(source: &str) -> ExecutionContex {
    let spirv = compile(source);
    let program = parse_words(spirv).unwrap();
    run(program)
}

pub fn get_var(program: &ExecutionContex, name: &str) -> RuntimeValue {
    program
        .mem_read(&program.find_valueid_for_name(name).unwrap())
        .unwrap()
}
