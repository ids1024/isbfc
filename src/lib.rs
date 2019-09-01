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

pub use crate::elf::{elf64_get_section, elf64_write};
pub use crate::parser::parse;
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
