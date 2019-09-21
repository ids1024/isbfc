use std::error::Error;
use std::fmt;
use unicode_width::UnicodeWidthStr;

#[derive(Clone)]
pub enum AST {
    Output,
    Input,
    Loop(Vec<AST>),
    Right,
    Left,
    Inc,
    Dec,
}

#[derive(Debug)]
pub enum ParseErrorType {
    UnclosedLoop,
    ExtraCloseLoop,
}
use ParseErrorType::*;

#[derive(Debug)]
pub struct ParseError {
    err: ParseErrorType,
    line: Vec<u8>,
    linenum: usize,
    offset: usize,
}

impl ParseError {
    fn new(err: ParseErrorType, code: &[u8], i: usize) -> Self {
        let (line, linenum, offset) = find_line(code, i);
        Self {
            err,
            line: line.into(),
            linenum,
            offset,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let line = String::from_utf8_lossy(&self.line);
        let width = UnicodeWidthStr::width(&line[0..self.offset]);

        match self.err {
            UnclosedLoop => {
                writeln!(f, "reached EOF with unterminated loop")?;
                writeln!(f, "Loop started at {}:{}", self.linenum, self.offset)?;
            }
            ExtraCloseLoop => {
                writeln!(
                    f,
                    "[ found at {}:{} when not in a loop",
                    self.linenum, self.offset
                )?;
            }
        };

        writeln!(f, "{}", line)?;
        write!(f, "{}^", " ".repeat(width))?;

        Ok(())
    }
}

impl Error for ParseError {}

/// Parses a string of brainfuck code to unoptimized AST
pub fn parse(code: &[u8]) -> Result<Vec<AST>, ParseError> {
    let mut i = 0;
    _parse(code, &mut i, 0)
}

fn _parse(code: &[u8], i: &mut usize, level: u32) -> Result<Vec<AST>, ParseError> {
    // Starting [ of the loop
    let start = i.saturating_sub(1);

    let mut tokens = Vec::new();
    while let Some(c) = code.get(*i) {
        *i += 1;

        match c {
            b'+' => tokens.push(AST::Inc),
            b'-' => tokens.push(AST::Dec),
            b'>' => tokens.push(AST::Right),
            b'<' => tokens.push(AST::Left),
            b'[' => tokens.push(AST::Loop(_parse(code, i, level + 1)?)),
            b']' => {
                return if level == 0 {
                    Err(ParseError::new(ExtraCloseLoop, code, *i - 1))
                } else {
                    Ok(tokens)
                };
            }
            b',' => tokens.push(AST::Input),
            b'.' => tokens.push(AST::Output),
            _ => (),
        };
    }

    if level != 0 {
        Err(ParseError::new(UnclosedLoop, code, start))
    } else {
        Ok(tokens)
    }
}

fn find_line(code: &[u8], i: usize) -> (&[u8], usize, usize) {
    let offset = code[0..i].iter().rev().take_while(|x| **x != b'\n').count();
    let end = i + code[i..].iter().take_while(|x| **x != b'\n').count();
    let linenum = code[0..(i - offset)]
        .iter()
        .filter(|x| **x == b'\n')
        .count();
    (&code[(i - offset)..end], linenum, offset)
}
