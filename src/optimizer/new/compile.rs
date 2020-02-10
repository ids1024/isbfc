use std::collections::HashMap;

use crate::{LIRBuilder, LIR};
use super::dag::Value;
use super::ir::IR;

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

fn ir_to_lir_iter(state: &mut CompileState, ir: &[IR]) {
    use crate::lir::prelude::*;

    let mut outbuffpos = 0;

    for i in ir {
        match i {
            IR::Output(offset) => {
                state
                    .lir
                    .mov(Buf("strbuf".into(), outbuffpos), Tape(*offset));
                outbuffpos += 1;
            }
            IR::Input(offset) => {
                state.lir.input("inputbuf", 0, 1);
                state.lir.mov(Tape(0), Buf("inputbuf".into(), 0));
            }
            IR::Loop(offset, inner, end_shift) => {
                if outbuffpos != 0 {
                    state.lir.output("strbuf", 0, outbuffpos);
                    if state.outbuffsize < outbuffpos + 1 {
                        state.outbuffsize = outbuffpos + 1;
                    }
                    outbuffpos = 0;
                }

                if *offset != 0 {
                    state.lir.shift(*offset);
                }

                state.loopnum += 1;
                let startlabel = format!("loop{}", state.loopnum);
                let endlabel = format!("endloop{}", state.loopnum);
                state.lir.jp(endlabel.clone());
                state.lir.label(startlabel.clone());

                ir_to_lir_iter(state, inner);
                if *end_shift != 0 {
                    state.lir.shift(*end_shift);
                }

                state.lir.label(endlabel.clone());
                state.lir.jnz(Tape(0), startlabel.clone());
            }
            IR::Expr(expr) => {
                let mut map: HashMap<_, RVal> = HashMap::new();

                for i in expr.topological_sort() {
                    match expr[i] {
                        Value::Tape(offset) => {
                            map.insert(i, Tape(offset).into());
                        }
                        Value::Const(value) => {
                            map.insert(i, Immediate(value));
                        }
                        Value::Multiply(a, b) => {
                            let reg = state.reg();
                            state.lir.mul(Reg(reg), map[&a].clone(), map[&b].clone());
                            map.insert(i, Reg(reg).into());
                        }
                        Value::Add(a, b) => {
                            let reg = state.reg();
                            state.lir.add(Reg(reg), map[&a].clone(), map[&b].clone());
                            map.insert(i, Reg(reg).into());
                        }
                    }
                }

                for (k, v) in expr.terminals() {
                    state.lir.mov(Tape(k), map[&v].clone());
                }
            }
        }
    }

    if outbuffpos != 0 {
        state.lir.output("strbuf".to_string(), 0, outbuffpos);
        if state.outbuffsize < outbuffpos + 1 {
            state.outbuffsize = outbuffpos + 1;
        }
    }
}

pub fn ir_to_lir(ir: &[IR]) -> Vec<LIR> {
    let mut state = CompileState::default();
    ir_to_lir_iter(&mut state, ir);
    state
        .lir
        .declare_bss_buf("strbuf".to_string(), state.outbuffsize);
    state.lir.declare_bss_buf("inputbuf".to_string(), 1);

    state.lir.build()
}
