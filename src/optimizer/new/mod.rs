// TODO: Not functional

#![allow(dead_code)]

use super::Optimizer;
use crate::{AST, LIR};
use std::io::Write;

mod dag;
mod compile;
use compile::ir_to_lir;
mod ir;
mod optimize_expr;
use optimize_expr::optimize_expr;

pub struct NewOptimizer;

impl Optimizer for NewOptimizer {
    fn optimize(&self, ast: &[AST], _level: u32) -> Vec<LIR> {
        let ir = optimize_expr(ast).0;
        ir_to_lir(&ir)
    }

    fn dumpir(&self, ast: &[AST], _level: u32, file: &mut dyn Write) -> std::io::Result<()> {
        let ir = optimize_expr(ast).0;
        write!(file, "{:#?}", ir)
    }
}
