//! Isbfc is an optimizing brainfuck compiler targeting x86_64 Linux
//!
//! # Examples
//! ```
//! extern crate isbfc;
//!
//! fn main() {
//!     // 2048 is the tape length to use
//!     let assembly = isbfc::parse(",[.,]").unwrap().optimize().compile(2048);
//!     print!("{}", assembly);
//! }
//! ```

use std::fmt;

#[macro_use]
mod macros;
mod compiler;
mod elf;
mod optimizer;
mod parser;
mod token;
mod assembly;
pub mod lir;
pub mod codegen_c;

pub use crate::elf::{elf64_get_section, elf64_write};
pub use crate::parser::{parse, AST};
pub use crate::token::Token;
pub use crate::assembly::{assemble, link};

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
