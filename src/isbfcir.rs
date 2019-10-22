use std::fmt;
use crate::{AST, Token};

/// Intermediate representation used by isbfc
pub struct IsbfcIR {
    /// Syntax tree of tokens
    pub tokens: Vec<Token>,
}

impl fmt::Debug for IsbfcIR {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.tokens.fmt(f)
    }
}

fn ast_to_tokens(ast: Vec<AST>) -> Vec<Token> {
    let mut tokens = Vec::new();
    for i in ast {
        match i {
            AST::Output => {
                tokens.push(Token::LoadOut(0, 0));
                tokens.push(Token::Output);
            },
            AST::Input => tokens.push(Token::Input),
            AST::Loop(inner) => tokens.push(Token::Loop(ast_to_tokens(inner))),
            AST::Right => tokens.push(Token::Move(1)),
            AST::Left => tokens.push(Token::Move(-1)),
            AST::Inc => tokens.push(Token::Add(0, 1)),
            AST::Dec => tokens.push(Token::Add(0, -1)),
        }
    }
    tokens
}

impl IsbfcIR {
    pub fn from_ast(ast: Vec<AST>) -> Self {
        IsbfcIR {
            tokens: ast_to_tokens(ast)
        }
    }
}
