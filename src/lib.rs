use std::iter::FromIterator;

pub mod token;
mod optimize_state;

use token::Token;
use token::Token::*;
use optimize_state::OptimizeState;

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

    for token in tokens {
        match *token {
            Set(..) | Add(..) | Move(_) | LoadOut(..) | LoadOutSet(_) | Output | MulCopy(..) => {}
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
            Set(mut offset, value) => {
                offset += state.shift;
                state.set(offset, value);
            }
            Add(mut offset, value) => {
                offset += state.shift;
                state.add(offset, value);
            }
            MulCopy(mut src, mut dest, mul) => {
                src += state.shift;
                dest += state.shift;
                if let Some(value) = state.sets.get(&src).cloned() {
                    state.add(dest, value * mul);
                } else {
                    if state.sets.contains_key(&dest) || state.adds.contains_key(&src) ||
                       state.adds.contains_key(&dest) {
                        state.apply_adds_sets();
                    }
                    state.tokens.push(MulCopy(src, dest, mul));
                }
            }
            If(offset, ref contents) => {
                let mut newcontents = Vec::new();
                for i in contents {
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
            for (offset, value) in &inner.sets {
                iftokens.push(Set(*offset, *value));
            }
            iftokens.push(Set(0, 0));
            outer.tokens.push(If(0, iftokens));
        } else {
            outer.set(0, 0);
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

        outer.set(0, 0);
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
