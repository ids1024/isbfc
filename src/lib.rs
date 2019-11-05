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

mod elf;
mod parser;
mod assembly;
mod optimizer;
pub mod lir;
pub mod codegen;

pub use crate::elf::{elf64_get_section, elf64_write};
pub use crate::parser::{parse, AST};
pub use crate::lir::LIR;
pub use crate::assembly::{assemble, link};
pub use crate::optimizer::{Optimizer, OldOptimizer};
