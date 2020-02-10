use crate::AST;
use super::dag::{Value, DAG};
use super::ir::IR;

pub fn optimize_expr(body: &[AST], outside_expr: &DAG) -> (Vec<IR>, i32) {
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
                expr.simplify();
                let (loop_body, loop_shift) = optimize_expr(body, &expr);
                if loop_body.len() == 1 && loop_shift == 0 {
                    if let IR::Expr(ref loop_expr) = loop_body[0] {
                        if let Some(mut new_expr) = optimize_expr_loop(&loop_expr) {
                            new_expr.shift(shift);
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

    expr.simplify();
    ir.push(IR::Expr(expr.clone()));

    (ir, shift)
}

/// Given a loop with no end shift, where the body is a single DAG,
/// if possible optimize such that the loop is replaced with a flat
/// DAG.
fn optimize_expr_loop(body_expr: &DAG) -> Option<DAG> {
    // TODO: Generalize constants to any tape offset unchange in DAG

    if body_expr.as_add_const(0) != Some(-1) {
        return None;
    }

    let mut expr = DAG::new(false);
    expr.set(0, Value::Const(0));

    for (k, v) in body_expr.terminals() {
        if k == 0 {
            continue;
        } else if body_expr[v] == Value::Tape(k) {
            continue;
        } else if let Some(a) = body_expr.as_add_const(k) {
            let tapeval = expr.add_node(Value::Tape(k));
            let lhs = expr.add_node(Value::Tape(0));
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
