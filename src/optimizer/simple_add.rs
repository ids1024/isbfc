use std::collections::HashMap;
use super::Optimizer;
use crate::{LIRBuilder, AST, LIR};
use std::io::Write;
use std::mem;

pub struct SimpleAddOptimizer;

impl Optimizer for SimpleAddOptimizer {
    fn optimize(&self, ast: &[AST], level: u32) -> Vec<LIR> {
        ir_to_lir(&ast_to_ir(ast))
    }

    fn dumpir(&self, ast: &[AST], level: u32, file: &mut dyn Write) -> std::io::Result<()> {
        writeln!(file, "{:#?}", ast_to_ir(ast))
    }
}

#[derive(Debug)]
enum SimpleAddIR {
    Output,
    Input,
    Loop(Vec<SimpleAddIR>),
    Adds(HashMap<i32, i32>),
    Shift(i32),
}

fn ast_to_ir(ast: &[AST]) -> Vec<SimpleAddIR> {
    let mut shift = 0;
    let mut ir = Vec::new();
    let mut adds = HashMap::new();

    for i in ast {
        match i {
            AST::Output => {
                ir.push(SimpleAddIR::Adds(mem::take(&mut adds)));
                ir.push(SimpleAddIR::Shift(shift));
                shift = 0;
                ir.push(SimpleAddIR::Output)
            }
            AST::Input => {
                ir.push(SimpleAddIR::Adds(mem::take(&mut adds)));
                ir.push(SimpleAddIR::Shift(shift));
                shift = 0;
                ir.push(SimpleAddIR::Input);
            }
            AST::Loop(inner) => {
                ir.push(SimpleAddIR::Adds(mem::take(&mut adds)));
                ir.push(SimpleAddIR::Shift(shift));
                shift = 0;
                ir.push(SimpleAddIR::Loop(ast_to_ir(inner)));
            }
            AST::Shift(offset) => { shift += offset; }
            AST::Add(add) => {
                *adds.entry(shift).or_insert(0) += *add;
            }
        }
    }

    ir.push(SimpleAddIR::Adds(mem::take(&mut adds)));
    ir.push(SimpleAddIR::Shift(shift));

    ir
}

#[derive(Default)]
struct CompileState {
    lir: LIRBuilder,
    loopnum: i32,
}

fn ir_to_lir(ir: &[SimpleAddIR]) -> Vec<LIR> {
    let mut state = CompileState::default();
    state.lir.declare_bss_buf("strbuf", 1);
    _ir_to_lir(ir, &mut state);
    state.lir.build()
}

fn _ir_to_lir(ir: &[SimpleAddIR], state: &mut CompileState) {
    use crate::lir::prelude::*;

    for i in ir {
        match i {
            SimpleAddIR::Output => {
                state.lir.mov(Buf("strbuf".into(), 0), Tape(0));
                state.lir.output("strbuf", 0, 1);
            }
            SimpleAddIR::Input => {
                state.lir.input("strbuf", 0, 1);
                state.lir.mov(Tape(0), Buf("strbuf".into(), 0));
            }
            SimpleAddIR::Loop(inner) => {
                state.loopnum += 1;
                let startlabel = format!("loop{}", state.loopnum);
                let endlabel = format!("endloop{}", state.loopnum);
                state.lir.jp(endlabel.clone());
                state.lir.label(startlabel.clone());

                _ir_to_lir(inner, state);

                state.lir.label(endlabel.clone());
                state.lir.jnz(Tape(0), startlabel.clone());
            }
            SimpleAddIR::Adds(adds) => {
                for (offset, value) in adds {
                    state.lir.add(Tape(*offset), Tape(*offset), Immediate(*value));
                }
            }
            SimpleAddIR::Shift(shift) => {
                state.lir.shift(*shift);
            }
        }
    }
}
