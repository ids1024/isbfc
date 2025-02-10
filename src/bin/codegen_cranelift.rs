use cranelift_codegen::ir::types::I64;
use isbfc::codegen::cranelift::codegen_fn;
use isbfc::{OldOptimizer, Optimizer};
use std::io::Read;

fn main() {
    let mut code = Vec::new();
    std::io::stdin().read_to_end(&mut code).unwrap();

    let ast = isbfc::parse(&code).unwrap();
    let lir = OldOptimizer.optimize(&ast, 3);
    let func = codegen_fn(&lir, I64, 8192);

    println!("{}", func.display());
}
