#![allow(dead_code)]

// Could use Cow<'static, String> instead of String? Won't that require &String?
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
    Buf(String, usize),
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum RVal {
    Reg(u32),
    Tape(i32),
    Buf(String, usize),
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

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum LIR {
    Shift(i32),
    Mul(LVal, RVal, RVal),
    Add(LVal, RVal, RVal),
    Sub(LVal, RVal, RVal),
    Mov(LVal, RVal),
    Label(String),
    Jp(String),
    Jz(RVal, String),
    Jnz(RVal, String),
    DeclareBssBuf(String, usize),
    Input(String, usize, usize),
    Output(String, usize, usize),
}

#[derive(Default)]
pub struct LIRBuilder {
    lir: Vec<LIR>,
}

impl LIRBuilder {
    pub fn new() -> Self {
        Self { lir: Vec::new() }
    }

    pub fn shift(&mut self, offset: i32) -> &mut Self {
        self.lir.push(LIR::Shift(offset));
        self
    }

    pub fn label(&mut self, name: String) -> &mut Self {
        self.lir.push(LIR::Label(name));
        self
    }

    pub fn jp(&mut self, name: String) -> &mut Self {
        self.lir.push(LIR::Jp(name));
        self
    }

    pub fn declare_bss_buf(&mut self, name: String, size: usize) -> &mut Self {
        self.lir.push(LIR::DeclareBssBuf(name, size));
        self
    }

    pub fn input(&mut self, name: String, offset: usize, size: usize) -> &mut Self {
        self.lir.push(LIR::Input(name, offset, size));
        self
    }

    pub fn output(&mut self, name: String, offset: usize, size: usize) -> &mut Self {
        self.lir.push(LIR::Output(name, offset, size));
        self
    }

    pub fn mul(&mut self, dest: LVal, a: impl Into<RVal>, b: impl Into<RVal>) -> &mut LIRBuilder {
        self.lir.push(LIR::Mul(dest, a.into(), b.into()));
        self
    }

    pub fn add(&mut self, dest: LVal, a: impl Into<RVal>, b: impl Into<RVal>) -> &mut LIRBuilder {
        self.lir.push(LIR::Add(dest, a.into(), b.into()));
        self
    }

    pub fn sub(&mut self, dest: LVal, a: impl Into<RVal>, b: impl Into<RVal>) -> &mut LIRBuilder {
        self.lir.push(LIR::Sub(dest, a.into(), b.into()));
        self
    }

    pub fn mov(&mut self, dest: LVal, src: impl Into<RVal>) -> &mut LIRBuilder {
        self.lir.push(LIR::Mov(dest, src.into()));
        self
    }

    pub fn jz(&mut self, comparand: impl Into<RVal>, name: String) -> &mut LIRBuilder {
        self.lir.push(LIR::Jz(comparand.into(), name));
        self
    }

    pub fn jnz(&mut self, comparand: impl Into<RVal>, name: String) -> &mut LIRBuilder {
        self.lir.push(LIR::Jnz(comparand.into(), name));
        self
    }

    pub fn build(self) -> Vec<LIR> {
        self.lir
    }
}
