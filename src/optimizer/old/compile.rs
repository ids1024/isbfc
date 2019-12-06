use super::token::Token;
use crate::lir::{self, LIRBuilder, LIR};

#[derive(Default)]
struct CompileState {
    lir: LIRBuilder,
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
    use lir::prelude::*;

    let mut outbuffpos = 0;
    for token in tokens {
        match *token {
            Token::Add(offset, value) => {
                state.lir.add(Tape(offset), Tape(offset), Immediate(value));
            }
            Token::MulCopy(src_idx, dest_idx, mult) => {
                let reg = state.reg();
                state.lir.mul(Reg(reg), Tape(src_idx), Immediate(mult));
                state.lir.add(Tape(dest_idx), Tape(dest_idx), Reg(reg));
            }
            Token::Set(offset, value) => {
                state.lir.mov(Tape(offset), Immediate(value));
            }
            Token::Move(offset) => {
                state.lir.shift(offset);
            }
            Token::Loop(ref content) => {
                state.loopnum += 1;
                let startlabel = format!("loop{}", state.loopnum);
                let endlabel = format!("endloop{}", state.loopnum);
                state.lir.jp(endlabel.clone());
                state.lir.label(startlabel.clone());

                compile_iter(state, &content);

                state.lir.label(endlabel.clone());
                state.lir.jnz(Tape(0), startlabel.clone());
            }
            Token::If(offset, ref content) => {
                state.ifnum += 1;
                let endlabel = format!("endif{}", state.ifnum);
                state.lir.jz(Tape(offset), endlabel.clone());

                compile_iter(state, &content);

                state.lir.label(endlabel.clone());
            }
            Token::Scan(offset) => {
                // Slighly more optimal than normal loop and move
                state.loopnum += 1;
                let startlabel = format!("loop{}", state.loopnum);
                let endlabel = format!("endloop{}", state.loopnum);
                state.lir.jp(endlabel.clone());
                state.lir.label(startlabel.clone());
                state.lir.shift(offset);
                state.lir.label(endlabel.clone());
                state.lir.jnz(Tape(0), startlabel.clone());
            }
            // XXX
            Token::Input => {
                state.lir.input("inputbuf", 0, 1);
                state.lir.mov(Tape(0), Buf("inputbuf".into(), 0));
            }
            Token::LoadOut(offset, addend) => {
                let reg = state.reg();
                state.lir.add(Reg(reg), Tape(offset), Immediate(addend));
                state.lir.mov(Buf("strbuf".into(), outbuffpos), Reg(reg));
                outbuffpos += 1;
            }
            Token::LoadOutSet(value) => {
                state
                    .lir
                    .mov(Buf("strbuf".into(), outbuffpos), Immediate(value));
                outbuffpos += 1;
            }
            Token::Output => {
                state.lir.output("strbuf", 0, outbuffpos);
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
    state.lir.declare_bss_buf("strbuf", state.outbuffsize);
    state.lir.declare_bss_buf("inputbuf", 1);
    state.lir.build()
}
