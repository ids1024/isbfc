use std::fmt;

use crate::AST;

/// A Token in isbfc's intermediate representation.
#[derive(Clone, PartialEq, Eq)]
pub enum Token {
    /// `Output` Writes output buffer to stdout
    Output,
    /// `Input` Reads one byte from stdin to the current cell
    Input,
    /// `Loop(content)` Runs *content* in loop while current cell is not zero
    Loop(Vec<Token>),
    /// `Move(offset)` Moves data pointer by *offset* cells
    Move(i32),
    /// `Add(offset, value)` Adds *value* to cell at *offset*
    Add(i32, i32),
    /// `Set(offset, value)` Sets cell at *offset* to *value*
    Set(i32, i32),
    /// `MulCopy(src, dest, mul)` Adds product of *mul* and the value at offset
    /// *src* to the cell at offset *dest*
    MulCopy(i32, i32, i32),
    /// `Scan(offset)` Equivalent to `Loop(Move(offset))`
    Scan(i32),
    /// `LoadOut(offset, add)` Appends the value of the cell at *offset* plus
    /// *add* to the output buffer
    LoadOut(i32, i32),
    /// `LoadOutSet(value)` Appends the constant value *value* to the output buffer
    LoadOutSet(i32),
    /// `LoadOutSet(offset, content)` Runs *content* if the cell at *offset* is not zero
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

pub fn ast_to_tokens(ast: &[AST]) -> Vec<Token> {
    let mut tokens = Vec::new();
    for i in ast {
        match i {
            AST::Output => {
                tokens.push(Token::LoadOut(0, 0));
                tokens.push(Token::Output);
            },
            AST::Input => tokens.push(Token::Input),
            AST::Loop(inner) => tokens.push(Token::Loop(ast_to_tokens(inner))),
            AST::Right => tokens.push(Token::Move(1)),
            AST::Left => tokens.push(Token::Move(-1)),
            AST::Inc => tokens.push(Token::Add(0, 1)),
            AST::Dec => tokens.push(Token::Add(0, -1)),
        }
    }
    tokens
}
