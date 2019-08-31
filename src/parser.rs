use crate::token::Token;
use crate::token::Token::*;
use crate::IsbfcIR;
use std::error::Error;
use std::fmt;
use unicode_width::UnicodeWidthStr;

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
                write!(f, "reached EOF with unterminated loop\n")?;
                write!(f, "Loop started at {}:{}\n", self.linenum, self.offset)?;
            }
            ExtraCloseLoop => {
                write!(
                    f,
                    "[ found at {}:{} when not in a loop\n",
                    self.linenum, self.offset
                )?;
            }
        };

        write!(f, "{}\n", line)?;
        write!(f, "{}^", " ".repeat(width))?;

        Ok(())
    }
}

impl Error for ParseError {}

/// Parses a string of brainfuck code to isbfc's intermediate representation,
/// without applying any optimization
pub fn parse(code: &[u8]) -> Result<IsbfcIR, ParseError> {
    let mut i = 0;
    _parse(code, &mut i, 0).map(|x| IsbfcIR { tokens: x })
}

fn _parse(code: &[u8], i: &mut usize, level: u32) -> Result<Vec<Token>, ParseError> {
    // Starting [ of the loop
    let start = i.saturating_sub(1);

    let mut tokens = Vec::new();
    while let Some(c) = code.get(*i) {
        *i += 1;

        match c {
            b'+' => tokens.push(Add(0, 1)),
            b'-' => tokens.push(Add(0, -1)),
            b'>' => tokens.push(Move(1)),
            b'<' => tokens.push(Move(-1)),
            b'[' => tokens.push(Loop(_parse(code, i, level + 1)?)),
            b']' => {
                return if level == 0 {
                    Err(ParseError::new(ExtraCloseLoop, code, *i - 1))
                } else {
                    Ok(tokens)
                };
            }
            b',' => tokens.push(Input),
            b'.' => {
                tokens.push(LoadOut(0, 0));
                tokens.push(Output);
            }
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
