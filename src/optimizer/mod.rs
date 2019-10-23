use std::io::Write;
use crate::{AST, LIR};

mod old;

pub use old::OldOptimizer;

pub trait Optimizer {
    fn optimize(ast: &[AST], level: u32) -> Vec<LIR>;
    fn dumpir(ast: &[AST], level: u32, file: &mut impl Write) -> std::io::Result<(())>;
}
