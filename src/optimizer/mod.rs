use std::io::Write;
use crate::{AST, LIR};

mod old;
mod simple;

pub use old::OldOptimizer;
pub use simple::SimpleOptimizer;

pub trait Optimizer {
    fn optimize(ast: &[AST], level: u32) -> Vec<LIR>;
    fn dumpir(ast: &[AST], level: u32, file: &mut impl Write) -> std::io::Result<(())>;
}
