extern crate isbfc;

use std::io::{Read, Write};
use isbfc::codegen::c_codegen::{codegen, CellType};

fn main() {
    let mut code = Vec::new();
    std::io::stdin().read_to_end(&mut code).unwrap();

    let ast = isbfc::parse(&code).unwrap();
    let lir = isbfc::optimizer::old::optimize(&ast, 3);
    let c = codegen(&lir, CellType::U64, 8192);

    std::io::stdout().write_all(c.as_bytes()).unwrap();
}
