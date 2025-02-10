use isbfc::codegen::c_codegen::{codegen, CellType};
use isbfc::{OldOptimizer, Optimizer};
use std::io::{Read, Write};

fn main() {
    let mut code = Vec::new();
    std::io::stdin().read_to_end(&mut code).unwrap();

    let ast = isbfc::parse(&code).unwrap();
    let lir = OldOptimizer.optimize(&ast, 3);
    let c = codegen(&lir, CellType::U64, 8192);

    std::io::stdout().write_all(c.as_bytes()).unwrap();
}
