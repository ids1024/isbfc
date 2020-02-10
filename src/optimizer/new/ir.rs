use crate::lir::RVal;
use super::dag::DAG;

#[derive(Debug)]
pub enum IR {
    Output(RVal),
    Input(i32),
    Loop(i32, Vec<IR>, i32),
    Expr(DAG),
}
