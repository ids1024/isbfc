use std::fmt;

mod token;
mod parser;
mod optimizer;
mod compiler;

pub use token::Token;
pub use parser::parse;


/// Intermediate representation used by isbfc
pub struct IsbfcIR {
    /// Syntax tree of tokens
    pub tokens: Vec<Token>
}

impl fmt::Debug for IsbfcIR {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.tokens.fmt(f)
    }
}
