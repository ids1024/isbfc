use std::io::Write;
use crate::{AST, LIR};
use super::Optimizer;

mod token;
mod optimize;
mod optimize_state;
mod compile;

pub struct OldOptimizer;

impl Optimizer for OldOptimizer {
    fn optimize(ast: &[AST], level: u32) -> Vec<LIR> {
        let mut tokens = token::ast_to_tokens(ast);
        if level > 0 {
            tokens = optimize::optimize(&tokens);
        }
        compile::compile(&tokens)
    }

    fn dumpir(ast: &[AST], level: u32, file: &mut impl Write) -> std::io::Result<(())> {
        let mut tokens = token::ast_to_tokens(ast);
        if level > 0 {
            tokens = optimize::optimize(&tokens);
        }
        writeln!(file, "{:#?}", tokens)
    }
}
