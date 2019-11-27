#![allow(dead_code)]

use std::borrow::Cow;

pub type CowStr = Cow<'static, str>;

// Need to consider fact that output buffer has 8-bit characters, while tape may not

pub mod prelude {
    use super::{LIRBuilder, LVal, RVal, LIR};
    pub use LVal::*;
    pub use RVal::Immediate;
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum LVal {
    Reg(u32),
    Tape(i32),
    Buf(CowStr, usize),
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum RVal {
    Reg(u32),
    Tape(i32),
    Buf(CowStr, usize),
    Immediate(i32),
}

impl PartialEq<RVal> for LVal {
    fn eq(&self, other: &RVal) -> bool {
        // TODO avoid clone
        &RVal::from(self.clone()) == other
    }
}

impl PartialEq<LVal> for RVal {
    fn eq(&self, other: &LVal) -> bool {
        // TODO avoid clone
        self == &RVal::from(other.clone())
    }
}

impl From<LVal> for RVal {
    fn from(lval: LVal) -> Self {
        match lval {
            LVal::Reg(num) => RVal::Reg(num),
            LVal::Tape(offset) => RVal::Tape(offset),
            LVal::Buf(name, offset) => RVal::Buf(name, offset),
        }
    }
}

/// LIR is isbfc's low level intermediate representation. In principle,
/// it serves a similar roles to LLVM IR, but it is much similer
/// and includes some Brainfuck specfic features.
///
/// Goals:
/// * Architecture agnostic
/// * Attempt to represent anything a Brainfuck compiler might want to
///   generate, without bias for a specific optimization design.
/// * Cell size agnostic
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum LIR {
    Shift(i32),
    Mul(LVal, RVal, RVal),
    Add(LVal, RVal, RVal),
    Sub(LVal, RVal, RVal),
    Mov(LVal, RVal),
    Label(CowStr),
    Jp(CowStr),
    Jz(RVal, CowStr),
    Jnz(RVal, CowStr),
    DeclareBssBuf(CowStr, usize),
    Input(CowStr, usize, usize),
    Output(CowStr, usize, usize),
}

#[derive(Default)]
pub struct LIRBuilder {
    lir: Vec<LIR>,
}

/// Defines a method that pushes a token to self.lir
macro_rules! pusher {
    ( $name:ident, $variant:ident, $( $arg:ident : $type:ty ),* ) => {
        pub fn $name(&mut self, $( $arg: $type ),*) -> &mut Self {
            self.lir.push(LIR::$variant($( $arg.into() ),*));
            self
        }
    }
}

impl LIRBuilder {
    pub fn new() -> Self {
        Self { lir: Vec::new() }
    }

    pusher!(shift, Shift, offset: i32);
    pusher!(label, Label, name: impl Into<CowStr>);
    pusher!(declare_bss_buf, DeclareBssBuf, name: impl Into<CowStr>, size: usize);
    pusher!(input, Input, name: impl Into<CowStr>, offset: usize, size: usize);
    pusher!(output, Output, name: impl Into<CowStr>, offset: usize, size: usize);
    pusher!(mul, Mul, dest: LVal, a: impl Into<RVal>, b: impl Into<RVal>);
    pusher!(add, Add, dest: LVal, a: impl Into<RVal>, b: impl Into<RVal>);
    pusher!(sub, Sub, dest: LVal, a: impl Into<RVal>, b: impl Into<RVal>);
    pusher!(mov, Mov, dest: LVal, src: impl Into<RVal>);
    pusher!(jp, Jp, name: impl Into<CowStr>);
    pusher!(jz, Jz, comparand: impl Into<RVal>, name: impl Into<CowStr>);
    pusher!(jnz, Jnz, comparand: impl Into<RVal>, name: impl Into<CowStr>);

    pub fn build(self) -> Vec<LIR> {
        self.lir
    }
}
