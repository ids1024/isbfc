// TODO: Not functional

#![allow(dead_code)]

use std::collections::HashMap;

use super::Optimizer;
use crate::{LIRBuilder, AST, LIR};
use std::io::Write;

mod dag;
use dag::{Value, DAG};

pub struct NewOptimizer;

impl Optimizer for NewOptimizer {
    fn optimize(&self, ast: &[AST], level: u32) -> Vec<LIR> {
        let ir = optimize_expr(ast, DAG::new(true)).0;
        ir_to_lir(&ir)
    }

    fn dumpir(&self, ast: &[AST], level: u32, file: &mut dyn Write) -> std::io::Result<()> {
        let ir = optimize_expr(ast, DAG::new(true)).0;
        write!(file, "{:#?}", ir)
    }
}

#[derive(Debug)]
enum IR {
    Output(i32),
    Input(i32),
    Loop(i32, Vec<IR>, i32),
    Expr(DAG),
}

fn optimize_expr(body: &[AST], outside_expr: DAG) -> (Vec<IR>, i32) {
    let mut ir = Vec::new();

    // TODO zeroing
    let mut expr = DAG::new(false);
    let mut shift = 0;
    for i in body {
        match i {
            AST::Input => {
                ir.push(IR::Input(shift));
                expr.set(shift, Value::Tape(shift));
            }
            AST::Output => {
                ir.push(IR::Expr(expr.clone()));
                expr.clear();
                ir.push(IR::Output(shift));
            }
            AST::Loop(body) => {
                let (loop_body, loop_shift) = optimize_expr(body, expr.clone());
                if loop_body.len() == 1 && shift == 0 {
                    if let IR::Expr(ref loop_expr) = loop_body[0] {
                        if let Some(new_expr) = optimize_expr_loop(shift, loop_expr.clone()) {
                            expr.extend(new_expr);
                            continue;
                        }
                    }
                }
                ir.push(IR::Expr(expr.clone()));
                expr.clear();
                expr.zeroed = false;
                ir.push(IR::Loop(shift, loop_body, loop_shift));
                shift = 0;
            }
            AST::Shift(offset) => {
                shift += offset;
            }
            AST::Add(add) => {
                expr.add(shift, *add);
            }
        }
    }

    ir.push(IR::Expr(expr.clone()));

    (ir, shift)
}

/// Given a loop with no end shift, where the body is a single DAG,
/// if possible optimize such that the loop is replaced with a flat
/// DAG.
fn optimize_expr_loop(shift: i32, body_expr: DAG) -> Option<DAG> {
    // TODO: Generalize constants to any tape offset unchange in DAG

    if body_expr.as_add_const(shift) != Some(-1) {
        return None;
    }

    let mut expr = DAG::new(false);
    expr.set(shift, Value::Const(0));

    for (k, v) in body_expr.terminals() {
        if k == shift {
            continue;
        } else if body_expr[v] == Value::Tape(k) {
            continue;
        } else if let Some(a) = body_expr.as_add_const(k) {
            let tapeval = expr.add_node(Value::Tape(k));
            let lhs = expr.add_node(Value::Tape(shift));
            let rhs = expr.add_node(Value::Const(a));
            let addend = expr.add_node(Value::Multiply(lhs, rhs));
            expr.set(k, Value::Add(tapeval, addend));
        } else if let Value::Const(a) = body_expr[v] {
            expr.set(k, Value::Const(a));
        } else {
            return None;
        }
    }

    Some(expr)
}

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

                state.lir.shift(*offset);

                state.loopnum += 1;
                let startlabel = format!("loop{}", state.loopnum);
                let endlabel = format!("endloop{}", state.loopnum);
                state.lir.jp(endlabel.clone());
                state.lir.label(startlabel.clone());

                ir_to_lir_iter(state, inner);
                state.lir.shift(*end_shift);

                state.lir.label(endlabel.clone());
                state.lir.jnz(Tape(0), startlabel.clone());
            }
            IR::Expr(expr) => {
                let mut map = HashMap::new();

                for i in expr.topological_sort() {
                    let reg = state.reg();
                    map.insert(i, reg);
                    match expr[i] {
                        Value::Tape(offset) => {
                            state.lir.mov(Reg(reg), Tape(offset));
                        }
                        Value::Const(value) => {
                            state.lir.mov(Reg(reg), Immediate(value));
                        }
                        Value::Multiply(a, b) => {
                            state.lir.mul(Reg(reg), Reg(map[&a]), Reg(map[&b]));
                        }
                        Value::Add(a, b) => {
                            state.lir.add(Reg(reg), Reg(map[&a]), Reg(map[&b]));
                        }
                    }
                }

                for (k, v) in expr.terminals() {
                    state.lir.mov(Tape(k), Reg(map[&v]));
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

fn ir_to_lir(ir: &[IR]) -> Vec<LIR> {
    let mut state = CompileState::default();
    ir_to_lir_iter(&mut state, ir);
    state
        .lir
        .declare_bss_buf("strbuf".to_string(), state.outbuffsize);
    state.lir.declare_bss_buf("inputbuf".to_string(), 1);

    state.lir.build()
}
