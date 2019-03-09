use std::str::Chars;
use crate::token::Token;
use crate::token::Token::*;
use crate::IsbfcIR;

#[derive(Debug)]
pub enum ParseError {
    UnclosedLoop,
    ExtraCloseLoop
}

/// Parses a string of brainfuck code to isbfc's intermediate representation,
/// without applying any optimization
pub fn parse(code: &str) -> Result<IsbfcIR, ParseError> {
    _parse(&mut code.chars(), 0).map(|x| IsbfcIR{tokens: x})
}

fn _parse(chars: &mut Chars, level: u32) -> Result<Vec<Token>, ParseError> {
    let mut tokens = Vec::new();
    while let Some(i) = chars.next() {
        match i {
            '+' => tokens.push(Add(0, 1)),
            '-' => tokens.push(Add(0, -1)),
            '>' => tokens.push(Move(1)),
            '<' => tokens.push(Move(-1)),
            '[' => tokens.push(Loop(_parse(chars, level+1)?)),
            ']' => {
                return if level == 0 {
                    Err(ParseError::ExtraCloseLoop)
                } else {
                    Ok(tokens)
                };
            }
            ',' => tokens.push(Input),
            '.' => {
                tokens.push(LoadOut(0, 0));
                tokens.push(Output);
            }
            _ => (),
        };
    }
    
    if level != 0 {
        Err(ParseError::UnclosedLoop)
    } else {
        Ok(tokens)
    }
}
