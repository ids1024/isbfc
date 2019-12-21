use crate::{AST, LIR};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::io::Write;

mod new;
mod old;
mod simple;
mod simple_add;

pub use new::NewOptimizer;
pub use old::OldOptimizer;
pub use simple::SimpleOptimizer;
pub use simple_add::SimpleAddOptimizer;

pub trait Optimizer: Sync {
    fn optimize(&self, ast: &[AST], level: u32) -> Vec<LIR>;
    fn dumpir(&self, ast: &[AST], level: u32, file: &mut dyn Write) -> std::io::Result<()>;
}

lazy_static! {
    pub static ref OPTIMIZERS: HashMap<&'static str, &'static dyn Optimizer> = {
        let mut m = HashMap::new();
        m.insert("old", &OldOptimizer as &dyn Optimizer);
        m.insert("simple", &SimpleOptimizer as &dyn Optimizer);
        m.insert("simple_add", &SimpleAddOptimizer as &dyn Optimizer);
        m.insert("new", &NewOptimizer as &dyn Optimizer);
        m
    };
}
