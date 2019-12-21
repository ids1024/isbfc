use super::Optimizer;
use crate::{AST, LIR};
use std::io::Write;

mod compile;
mod optimize;
mod optimize_state;
mod token;

pub struct OldOptimizer;

impl Optimizer for OldOptimizer {
    fn optimize(&self, ast: &[AST], level: u32) -> Vec<LIR> {
        let mut tokens = token::ast_to_tokens(ast);
        if level > 0 {
            tokens = optimize::optimize(&tokens);
        }
        compile::compile(&tokens)
    }

    fn dumpir(&self, ast: &[AST], level: u32, file: &mut dyn Write) -> std::io::Result<()> {
        let mut tokens = token::ast_to_tokens(ast);
        if level > 0 {
            tokens = optimize::optimize(&tokens);
        }
        writeln!(file, "{:#?}", tokens)
    }
}
