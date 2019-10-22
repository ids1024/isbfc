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

#[macro_use]
mod macros;
mod elf;
mod optimizer;
mod parser;
mod token;
mod assembly;
mod isbfcir;
pub mod lir;
pub mod codegen;

pub use crate::elf::{elf64_get_section, elf64_write};
pub use crate::parser::{parse, AST};
pub use crate::token::Token;
pub use crate::assembly::{assemble, link};
pub use crate::isbfcir::IsbfcIR;
