use crate::{AST, LIR};

mod token;
mod optimize;
mod optimize_state;
mod compile;

pub fn optimize(ast: &[AST], level: u32) -> Vec<LIR> {
    let mut tokens = token::ast_to_tokens(ast);
    if level > 0 {
        tokens = optimize::optimize(&tokens);
    }
    compile::compile(&tokens)
}
