// TODO: Not functional

#![allow(dead_code)]

use std::collections::{BTreeSet, HashMap, VecDeque};
use std::fmt;

use super::Optimizer;
use crate::{LIRBuilder, AST, LIR};
use std::io::Write;

pub struct NewOptimizer;

impl Optimizer for NewOptimizer {
    fn optimize(&self, ast: &[AST], level: u32) -> Vec<LIR> {
        let ir = optimize_expr(ast, DAG::new(true)).0;
        ir_to_lir(&ir)
    }

    fn dumpir(&self, ast: &[AST], level: u32, file: &mut dyn Write) -> std::io::Result<(())> {
        let ir = optimize_expr(ast, DAG::new(true)).0;
        write!(file, "{:#?}", ir)
    }
}

#[derive(Clone, Copy, Hash)]
enum Value {
    Tape(i32),
    Const(i32),
    Multiply(usize, usize),
    Add(usize, usize),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Tape(offset) => {
                write!(f, "tape[{}]", offset)?;
            }
            Value::Const(value) => {
                write!(f, "{}", value)?;
            }
            Value::Multiply(ref l, ref r) => {
                write!(f, "({:?} * {:?})", l, r)?;
            }
            Value::Add(ref l, ref r) => {
                write!(f, "({:?} + {:?})", l, r)?;
            }
        }
        Ok(())
    }
}

// TODO: try adding HashMap<Value, usize> for reverse node lookup;
// see if this helps for efficiency.
#[derive(Clone, Debug)]
struct DAG {
    nodes: Vec<Value>,
    terminals: HashMap<i32, usize>,
    zeroed: bool,
}

impl DAG {
    fn new(zeroed: bool) -> Self {
        Self {
            nodes: Vec::new(),
            terminals: HashMap::new(),
            zeroed,
        }
    }

    fn add_node(&mut self, value: Value) -> usize {
        self.nodes.push(value);
        self.nodes.len() - 1
    }

    fn default_value(&self, offset: i32) -> Value {
        if self.zeroed {
            Value::Const(0)
        } else {
            Value::Tape(offset)
        }
    }

    fn set(&mut self, offset: i32, value: Value) {
        let node = self.add_node(value);
        self.terminals.insert(offset, node);
    }

    fn get(&self, offset: i32) -> Value {
        self.terminals
            .get(&offset)
            .map(|x| self.nodes[*x])
            .unwrap_or(self.default_value(offset))
    }

    fn get_node(&mut self, offset: i32) -> usize {
        if let Some(node) = self.terminals.get(&offset) {
            *node
        } else {
            let node = self.add_node(self.default_value(offset));
            self.terminals.insert(offset, node);
            node
        }
    }

    fn add(&mut self, offset: i32, value: Value) {
        let old_node = self.get_node(offset);
        let new_node = self.add_node(value);
        self.set(offset, Value::Add(old_node, new_node));
    }

    fn clear(&mut self) {
        self.nodes.clear();
        self.terminals.clear();
    }

    fn shift(&mut self, shift: i32) {
        for i in self.nodes.iter_mut() {
            if let Value::Tape(offset) = *i {
                *i = Value::Tape(offset + shift);
            }
        }
    }

    fn topological_sort(&self) -> impl Iterator<Item = usize> {
        // Assumes nodes are never deleted, so numberic order is toplogical
        /*
        let mut set: BTreeSet<usize> = BTreeSet::new();
        let mut queue: VecDeque<usize> = VecDeque::new();
        set.extend(self.terminals.values().cloned());
        queue.extend(self.terminals.values().cloned());
        while let Some(i) = queue.pop_front() {
            match self.nodes[i] {
                Value::Multiply(a, b) | Value::Add(a, b) => {
                    if !set.contains(&a) {
                        queue.push_back(a);
                        set.insert(a);
                    }
                    if !set.contains(&b) {
                        queue.push_back(b);
                        set.insert(b);
                    }
                }
                Value::Tape(_) | Value::Const(_) => {}
            }
        }
        set.into_iter()
        */
        // TODO: doesn't skip unneeded nodes
        0..self.nodes.len()
    }
    //fn append(&mut self, expr: CalcExpr);
    //fn simplify(&mut self);
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

    let mut expr = DAG::new(outside_expr.zeroed);
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
                /*
                if loop_body.len() == 1 && shift == 0 {
                    if let IR::Expr(ref loop_expr) = loop_body[0] {
                        expr.append(optimize_expr_loop(loop_expr.clone()));
                        continue;
                    }
                }
                */
                ir.push(IR::Expr(expr.clone()));
                expr.clear();
                expr.zeroed = false;
                shift = 0;
                ir.push(IR::Loop(shift, loop_body, loop_shift));
            }
            AST::Shift(offset) => {
                shift += offset;
            }
            AST::Add(add) => {
                expr.add(shift, Value::Const(*add));
            }
        }
    }

    ir.push(IR::Expr(expr.clone()));

    (ir, shift)
}

/*
fn optimize_expr_loop(body_expr: DAG) -> DAG {
    let val = body_expr.get(0);
    // Only works when adding const to tape[0]?
    // Total number of iterations:
    //    Value::Tape(0) -
    // TODO: Detect infinite loop
    for (_offset, _value) in &body_expr.map {
        //outside_expr.set(offset,
    }
    body_expr // FIXME
}
*/

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

                state.loopnum += 1;
                let startlabel = format!("loop{}", state.loopnum);
                let endlabel = format!("endloop{}", state.loopnum);
                state.lir.jp(endlabel.clone());
                state.lir.label(startlabel.clone());

                ir_to_lir_iter(state, inner);
                state.lir.shift(*end_shift);

                state.lir.label(endlabel.clone());
                state.lir.jnz(Tape(*offset), startlabel.clone());
            }
            IR::Expr(expr) => {
                let mut map = HashMap::new();

                for i in expr.topological_sort() {
                    let reg = state.reg();
                    map.insert(i, reg);
                    match expr.nodes[i] {
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

                for (k, v) in &expr.terminals {
                    state.lir.mov(Tape(*k), Reg(map[v]));
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
