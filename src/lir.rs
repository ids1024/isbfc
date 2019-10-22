#![allow(dead_code)]

// Could use Cow<'static, String> instead of String? Won't that require &String?
// Need to consider fact that output buffer has 8-bit characters, while tape may not

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

// Shorthand notation for constructing LIR
pub mod lir {
    use super::{LVal, RVal, LIR};

    pub use LVal::*;
    pub use RVal::Immediate;

    pub use LIR::Shift as shift;
    pub use LIR::Label as label;
    pub use LIR::Jp as jp;
    pub use LIR::DeclareBssBuf as declare_bss_buf;
    pub use LIR::Input as input;
    pub use LIR::Output as output;

    pub fn mul(dest: LVal, a: impl Into<RVal>, b: impl Into<RVal>) -> LIR {
        LIR::Mul(dest, a.into(), b.into())
    }

    pub fn add(dest: LVal, a: impl Into<RVal>, b: impl Into<RVal>) -> LIR {
        LIR::Add(dest, a.into(), b.into())
    }

    pub fn sub(dest: LVal, a: impl Into<RVal>, b: impl Into<RVal>) -> LIR {
        LIR::Sub(dest, a.into(), b.into())
    }

    pub fn mov(dest: LVal, src: impl Into<RVal>) -> LIR {
        LIR::Mov(dest, src.into())
    }

    pub fn jz(comparand: impl Into<RVal>, name: String) -> LIR {
        LIR::Jz(comparand.into(), name)
    }

    pub fn jnz(comparand: impl Into<RVal>, name: String) -> LIR {
        LIR::Jnz(comparand.into(), name)
    }
}
