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

mod assembly;
pub mod codegen;
mod elf;
pub mod lir;
mod optimizer;
mod parser;

pub use crate::assembly::{assemble, link};
pub use crate::elf::{elf64_get_section, elf64_write};
pub use crate::lir::{LIRBuilder, LIR};
pub use crate::optimizer::{OldOptimizer, Optimizer, SimpleOptimizer, SimpleAddOptimizer, OPTIMIZERS};
pub use crate::parser::{parse, AST};
