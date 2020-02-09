use super::dag::DAG;

#[derive(Debug)]
pub enum IR {
    Output(i32),
    Input(i32),
    Loop(i32, Vec<IR>, i32),
    Expr(DAG),
}
