use std::collections::BTreeMap;
use std::fmt;
use std::iter::FromIterator;

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

use Token::*;

#[derive(Default)]
struct OptimizeState {
    tokens: Vec<Token>,
    // With HashMap, the order sometimes switches
    // in recursion, and the optimizer never exits.
    adds: BTreeMap<i32, i32>,
    sets: BTreeMap<i32, i32>,
    shift: i32,
}

impl OptimizeState {
    fn apply_shift(&mut self) {
        if self.shift != 0 {
            self.tokens.push(Move(self.shift));
            self.shift = 0;
        }
    }

    fn apply_adds_sets(&mut self) {
        for (offset, value) in self.sets.iter() {
            self.tokens.push(Set(*offset, *value));
        }
        for (offset, value) in self.adds.iter() {
            self.tokens.push(Add(*offset, *value));
        }
        self.sets.clear();
        self.adds.clear();
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

fn _optimize(tokens: &Vec<Token>) -> OptimizeState {
    let mut do_output = false;
    let mut state = OptimizeState::default();

    for token in tokens.iter() {
        match *token {
            Set(..) | Add(..) | Move(_) | LoadOut(..) | LoadOutSet(_) | Output => {}
            _ => {
                if do_output {
                    state.tokens.push(Output);
                    do_output = false;
                }

                state.apply_adds_sets();
            }
        }

        match *token {
            Loop(_) | Input | Scan(_) => {
                state.apply_shift();
            }
            _ => {}
        }

        match *token {
            Set(mut offset, val) => {
                offset += state.shift;
                // Add before Set does nothing; remove it
                state.adds.remove(&offset);
                state.sets.insert(offset, val);
            }
            Add(mut offset, mut val) => {
                offset += state.shift;
                if state.sets.contains_key(&offset) {
                    val = state.sets.get(&offset).unwrap() + val;
                    state.sets.insert(offset, val);
                } else {
                    val = state.adds.get(&offset).unwrap_or(&0) + val;
                    state.adds.insert(offset, val);
                }
            }
            MulCopy(src, dest, mul) => {
                state.tokens.push(MulCopy(src + state.shift, dest + state.shift, mul))
            }
            If(offset, ref contents) => {
                let mut newcontents = Vec::new();
                for i in contents.iter() {
                    newcontents.push(match *i {
                        Set(offset, value) => Set(offset + state.shift, value),
                        MulCopy(src, dest, mul) => {
                            MulCopy(src + state.shift, dest + state.shift, mul)
                        }
                        _ => unreachable!(),
                    });
                }
                state.tokens.push(If(offset + state.shift, newcontents));
            }
            Move(offset) => state.shift += offset,
            Output => do_output = true,
            LoadOut(mut offset, add) => {
                offset += state.shift;
                if state.sets.contains_key(&offset) {
                    state.tokens.push(LoadOutSet(state.sets.get(&offset).unwrap() + add));
                } else {
                    state.tokens.push(LoadOut(offset, state.adds.get(&offset).unwrap_or(&0) + add));
                }
            }
            Loop(ref contents) => _optimize_loop(contents, &mut state),
            LoadOutSet(value) => state.tokens.push(LoadOutSet(value)),
            Input => state.tokens.push(Input),
            Scan(offset) => state.tokens.push(Scan(offset + state.shift)),
        }
    }

    if do_output {
        state.tokens.push(Output);
    }

    state
}

fn _optimize_loop(tokens: &Vec<Token>, outer: &mut OptimizeState) {
    let mut inner = _optimize(tokens);

    if inner.shift != 0 && inner.sets.is_empty() && inner.adds.is_empty() &&
       inner.tokens.is_empty() {
        outer.tokens.push(Scan(inner.shift));
    } else if inner.shift == 0 && inner.tokens.is_empty() && inner.adds.contains_key(&0) &&
              inner.adds.len() == 1 {
        if !inner.sets.is_empty() {
            let mut iftokens = Vec::new();
            for (offset, value) in inner.sets.iter() {
                iftokens.push(Set(*offset, *value));
            }
            iftokens.push(Set(0, 0));
            outer.tokens.push(If(0, iftokens));
        } else {
            outer.sets.insert(0, 0);
        }
    } else if inner.shift == 0 && inner.tokens.is_empty() && inner.adds.get(&0) == Some(&-1) {
        let contents = inner.adds.iter().filter_map(|(offset, value)| {
            if *offset != 0 {
                let src = 0;
                let dest = *offset;
                let mul = *value;
                Some(MulCopy(src, dest, mul))
            } else {
                None
            }
        });

        if !inner.sets.is_empty() {
            let iftokens = Vec::from_iter(inner.sets
                .iter()
                .map(|(offset, value)| Set(*offset, *value))
                .chain(contents));
            outer.tokens.push(If(0, iftokens));
        } else {
            outer.tokens.extend(contents);
        }

        outer.sets.insert(0, 0);
    } else {
        inner.apply_adds_sets();
        inner.apply_shift();

        outer.tokens.push(Loop(inner.tokens));
    }
}

pub fn optimize(tokens: Vec<Token>) -> Vec<Token> {
    let mut oldtokens = tokens;
    // Ignore sets/adds/shifts at end of file
    let mut newtokens = _optimize(&oldtokens).tokens;
    while newtokens != oldtokens {
        oldtokens = newtokens;
        newtokens = _optimize(&oldtokens).tokens;
    }
    newtokens
}
