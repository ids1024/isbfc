use crate::lir::{self, LIR};
use super::token::Token;

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
    use lir::lir::*;

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
            Token::Input => {
                state.lir.push(input("inputbuf".to_string(), 0, 1));
                state.lir.push(mov(Tape(0), Buf("inputbuf".to_string(), 0)));
            }
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
    state.lir.push(lir::lir::declare_bss_buf("strbuf".to_string(), state.outbuffsize));
    state.lir.push(lir::lir::declare_bss_buf("inputbuf".to_string(), 1));
    state.lir
}
