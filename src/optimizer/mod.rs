use crate::{AST, LIR};
use std::io::Write;

mod old;
mod simple;

pub use old::OldOptimizer;
pub use simple::SimpleOptimizer;

pub trait Optimizer {
    fn optimize(ast: &[AST], level: u32) -> Vec<LIR>;
    fn dumpir(ast: &[AST], level: u32, file: &mut impl Write) -> std::io::Result<(())>;
}
