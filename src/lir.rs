#![allow(dead_code)]

// Could use Cow<'static, String> instead of String? Won't that require &String?
// Need to consider fact that output buffer has 8-bit characters, while tape may not

use crate::token::Token;

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

#[derive(Default)]
struct CompileState {
    lir: Vec<LIR>,
    loopnum: i32,
    ifnum: i32,
    outbuffsize: usize,
    regnum: u32,
}

impl CompileState {
    /// Allocate a new, unique register (for SSA output)
    fn reg(&mut self) -> u32 {
        let r = self.regnum;
        self.regnum += 1;
        r
    }
}

fn compile_iter(state: &mut CompileState, tokens: &[Token]) {
    use lir::*;

    let mut outbuffpos = 0;
    for token in tokens {
        match *token {
            Token::Add(offset, value) => state.lir.push(add(Tape(offset), Tape(offset), Immediate(value))),
            Token::MulCopy(src_idx, dest_idx, mult) => {
                let reg = state.reg();
                state.lir.push(mul(Reg(reg), Tape(src_idx), Immediate(mult)));
                state.lir.push(add(Tape(dest_idx), Tape(dest_idx), Reg(reg)));
            }
            Token::Set(offset, value) => state.lir.push(mov(Tape(offset), Immediate(value))),
            Token::Move(offset) => state.lir.push(shift(offset)),
            Token::Loop(ref content) => {
                state.loopnum += 1;
                let startlabel = format!("loop{}", state.loopnum);
                let endlabel = format!("endloop{}", state.loopnum);
                state.lir.push(jp(endlabel.clone()));
                state.lir.push(label(startlabel.clone()));

                compile_iter(state, &content);

                state.lir.push(label(endlabel.clone()));
                state.lir.push(jnz(Tape(0), startlabel.clone()));
            }
            Token::If(offset, ref content) => {
                state.ifnum += 1;
                let endlabel = format!("endif{}", state.ifnum);
                state.lir.push(jz(Tape(offset), endlabel.clone()));

                compile_iter(state, &content);

                state.lir.push(label(endlabel.clone()));
            }
            Token::Scan(offset) => {
                // Slighly more optimal than normal loop and move
                state.loopnum += 1;
                let startlabel = format!("loop{}", state.loopnum);
                let endlabel = format!("endloop{}", state.loopnum);
                state.lir.push(jp(endlabel.clone()));
                state.lir.push(label(startlabel.clone()));
                state.lir.push(shift(offset));
                state.lir.push(label(endlabel.clone()));
                state.lir.push(jnz(Tape(0), startlabel.clone()));
            }
            // XXX
            Token::Input => state.lir.push(input("strbuf".to_string(), 0, outbuffpos)),
            Token::LoadOut(offset, addend) => {
                let reg = state.reg();
                state.lir.push(add(Reg(reg), Tape(offset), Immediate(addend)));
                state.lir.push(mov(Buf("strbuf".to_string(), outbuffpos), Reg(reg)));
                outbuffpos += 1;
            }
            Token::LoadOutSet(value) => {
                state.lir.push(mov(Buf("strbuf".to_string(), outbuffpos), Immediate(value)));
                outbuffpos += 1;
            }
            Token::Output => {
                state.lir.push(output("strbuf".to_string(), 0, outbuffpos));
                if state.outbuffsize < outbuffpos + 1 {
                    state.outbuffsize = outbuffpos + 1;
                }
                outbuffpos = 0;
            }
        }
    }
}

pub fn compile(tokens: &[Token]) -> Vec<LIR> {
    let mut state = CompileState::default();
    compile_iter(&mut state, tokens);
    state.lir.push(lir::declare_bss_buf("strbuf".to_string(), state.outbuffsize));
    state.lir
}
