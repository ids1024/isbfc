use std::str::Chars;
use token::Token;
use token::Token::*;
use IsbfcIR;

/// Parses a string of brainfuck code to isbfc's intermediate representation,
/// without applying any optimization
pub fn parse(code: &str) -> IsbfcIR {
    let tokens = _parse(&mut code.chars());
    IsbfcIR{tokens: tokens}
}

fn _parse(chars: &mut Chars) -> Vec<Token> {
    let mut tokens = Vec::new();
    while let Some(i) = chars.next() {
        match i {
            '+' => tokens.push(Add(0, 1)),
            '-' => tokens.push(Add(0, -1)),
            '>' => tokens.push(Move(1)),
            '<' => tokens.push(Move(-1)),
            '[' => tokens.push(Loop(_parse(chars))),
            ']' => {
                break;
            }
            ',' => tokens.push(Input),
            '.' => {
                tokens.push(LoadOut(0, 0));
                tokens.push(Output);
            }
            _ => (),
        };
    }

    tokens
}
