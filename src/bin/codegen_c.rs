extern crate isbfc;

use std::io::{Read, Write};

fn main() {
    let mut code = Vec::new();
    std::io::stdin().read_to_end(&mut code).unwrap();

    let ast = isbfc::parse(&code).unwrap();
    let mut ir = isbfc::IsbfcIR::from_ast(ast);
    ir = ir.optimize();
    let lir = isbfc::lir::compile(&ir.tokens);
    let c = isbfc::codegen_c::codegen(&lir);

    std::io::stdout().write_all(c.as_bytes()).unwrap();
}
