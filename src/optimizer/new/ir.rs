use super::dag::DAG;
use crate::lir::RVal;

#[derive(Debug)]
pub enum IR {
    Output(RVal),
    Input(i32),
    Loop(i32, Vec<IR>, i32),
    Expr(DAG),
}
