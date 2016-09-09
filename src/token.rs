use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub enum Token {
    Output,
    Input,
    Loop(Vec<Token>),
    Move(i32),
    Add(i32, i32),
    Set(i32, i32),
    MulCopy(i32, i32, i32),
    Scan(i32),
    LoadOut(i32, i32),
    LoadOutSet(i32),
    If(i32, Vec<Token>),
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Token::Output => write!(f, "Output"),
            Token::Input => write!(f, "Input"),
            Token::Move(offset) => write!(f, "Move(offset={})", offset),
            Token::Add(offset, value) => write!(f, "Add(offset={}, value={})", offset, value),
            Token::Set(offset, value) => write!(f, "Set(offset={}, value={})", offset, value),
            Token::MulCopy(src, dest, mul) => {
                write!(f, "MulCopy(src={}, dest={}, mul={})", src, dest, mul)
            }
            Token::Scan(offset) => write!(f, "Scan(offset={})", offset),
            Token::LoadOut(offset, add) => write!(f, "LoadOut(offset={}, add={})", offset, add),
            Token::LoadOutSet(value) => write!(f, "LoadOutSet(value={})", value),
            Token::Loop(ref content) => {
                if f.alternate() {
                    write!(f, "Loop(content={:#?})", content)
                } else {
                    write!(f, "Loop(content={:?})", content)
                }
            }
            Token::If(offset, ref content) => {
                if f.alternate() {
                    write!(f, "If(offset={}, content={:#?})", offset, content)
                } else {
                    write!(f, "If(offset={}, content={:?})", offset, content)
                }
            }
        }
    }
}
