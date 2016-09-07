use std::collections::BTreeMap;
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
use Token::*;

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Output => write!(f, "Output"),
            Input => write!(f, "Input"),
            Move(offset) => write!(f, "Move(offset={})", offset),
            Add(offset, value) => write!(f, "Add(offset={}, value={})", offset, value),
            Set(offset, value) => write!(f, "Set(offset={}, value={})", offset, value),
            MulCopy(src, dest, mul) => {
                write!(f, "MulCopy(src={}, dest={}, mul={})", src, dest, mul)
            }
            Scan(offset) => write!(f, "Scan(offset={})", offset),
            LoadOut(offset, add) => write!(f, "LoadOut(offset={}, add={})", offset, add),
            LoadOutSet(value) => write!(f, "LoadOutSet(value={})", value),
            Loop(ref content) => write!(f, "Loop(content={:?})", content),
            If(offset, ref content) => write!(f, "If(offset={}, content={:?})", offset, content),
        }
    }
}

pub fn parse(code: &str) -> Vec<Token> {
    _parse(&mut code.chars())
}

fn _parse(chars: &mut std::str::Chars) -> Vec<Token> {
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

fn _optimize(tokens: &Vec<Token>) -> Vec<Token> {
    let mut newtokens: Vec<Token> = Vec::with_capacity(tokens.len());
    let mut shift = 0;
    let mut do_output = false;
    // With HashMap, the order sometimes switches
    // in recursion, and the optimizer never exits.
    let mut adds: BTreeMap<i32, i32> = BTreeMap::new();
    let mut sets: BTreeMap<i32, i32> = BTreeMap::new();

    for token in tokens.iter() {
        match *token {
            Set(..) | Add(..) | Move(_) | LoadOut(..) | LoadOutSet(_) | Output => {}
            _ => {
                if do_output {
                    newtokens.push(Output);
                    do_output = false;
                }

                for (offset, value) in sets.iter() {
                    newtokens.push(Set(*offset, *value));
                }
                for (offset, value) in adds.iter() {
                    newtokens.push(Add(*offset, *value));
                }
                sets.clear();
                adds.clear();
            }
        }

        if shift != 0 {
            match *token {
                Loop(_) | Input | Scan(_) => {
                    newtokens.push(Move(shift));
                    shift = 0;
                }
                _ => {}
            }
        }

        match *token {
            Set(mut offset, val) => {
                offset += shift;
                // Add before Set does nothing; remove it
                adds.remove(&offset);
                sets.insert(offset, val);
            }
            Add(mut offset, mut val) => {
                offset += shift;
                if sets.contains_key(&offset) {
                    val = sets.get(&offset).unwrap() + val;
                    sets.insert(offset, val);
                } else {
                    val = adds.get(&offset).unwrap_or(&0) + val;
                    adds.insert(offset, val);
                }
            }
            MulCopy(src, dest, mul) => newtokens.push(MulCopy(src + shift, dest + shift, mul)),
            // XXX Deal with shift in if, if those are ever generated
            If(offset, ref contents) => newtokens.push(If(offset + shift, _optimize(contents))),
            Move(offset) => shift += offset,
            Output => do_output = true,
            LoadOut(mut offset, add) => {
                offset += shift;
                if sets.contains_key(&offset) {
                    newtokens.push(LoadOutSet(sets.get(&offset).unwrap() + add));
                } else {
                    newtokens.push(LoadOut(offset, adds.get(&offset).unwrap_or(&0) + add));
                }
            }
            Loop(ref contents) => {
                newtokens.push(Loop(_optimize(contents)))
            }
            LoadOutSet(value) => newtokens.push(LoadOutSet(value)),
            Input => newtokens.push(Input),
            Scan(offset) => newtokens.push(Scan(offset + shift)),
        }
    }

    if do_output {
        newtokens.push(Output);
    }
    for (offset, value) in sets.iter() {
        newtokens.push(Set(*offset, *value));
    }
    for (offset, value) in adds.iter() {
        newtokens.push(Add(*offset, *value));
    }
    if shift != 0 {
        newtokens.push(Move(shift));
    }

    newtokens
}

pub fn optimize(tokens: Vec<Token>) -> Vec<Token> {
    let mut oldtokens = tokens;
    let mut newtokens = _optimize(&oldtokens);
    while newtokens != oldtokens {
	oldtokens = newtokens;
        newtokens = _optimize(&oldtokens);
    }
    newtokens
}
