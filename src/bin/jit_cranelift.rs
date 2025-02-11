use cranelift_codegen::ir::types::I64;
use cranelift_codegen::ir::{Signature};
use cranelift_codegen::isa::CallConv;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataDescription, DataId, FuncId, Module};
use isbfc::codegen::cranelift::codegen_fn;
use isbfc::{OldOptimizer, Optimizer};
use std::io::Read;

fn main() {
    // TODO read from file
    let mut code = Vec::new();
    std::io::stdin().read_to_end(&mut code).unwrap();

    let ast = isbfc::parse(&code).unwrap();
    let lir = OldOptimizer.optimize(&ast, 3);
    let mut func = codegen_fn(&lir, I64, 8192);

    let builder = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();
    let mut module = JITModule::new(builder);

    //let data_id = DataId::from_u32(0);
    let data_id = module.declare_data("tape", cranelift_module::Linkage::Local, true, false).unwrap();
    let mut data_desc = DataDescription::new();
    data_desc.define_zeroinit(8192 * 8);
    module.define_data(data_id, &data_desc).unwrap();
    module.declare_data_in_func(data_id, &mut func);

    //let func_id = FuncId::from_u32(0);
    let signature = Signature::new(CallConv::SystemV);
    let func_id = module.declare_function("main", cranelift_module::Linkage::Local, &signature).unwrap();
    //let mut context = module.make_context();
    let mut context = cranelift_codegen::Context::for_function(func);
    // TODO populate context
    module.define_function(func_id, &mut context).unwrap();
    module.finalize_definitions().unwrap();
    // TODO jump to fn?
    module.get_finalized_function(func_id);

}
