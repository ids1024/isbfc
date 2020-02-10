// TODO: Not functional

#![allow(dead_code)]

use super::Optimizer;
use crate::{AST, LIR};
use std::io::Write;

mod dag;
mod compile;
use compile::ir_to_lir;
mod ir;
mod optimize;
use optimize::optimize;

pub struct NewOptimizer;

impl Optimizer for NewOptimizer {
    fn optimize(&self, ast: &[AST], _level: u32) -> Vec<LIR> {
        ir_to_lir(&optimize(ast))
    }

    fn dumpir(&self, ast: &[AST], _level: u32, file: &mut dyn Write) -> std::io::Result<()> {
        write!(file, "{:#?}", optimize(ast))
    }
}
