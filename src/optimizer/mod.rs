use std::iter::FromIterator;

mod optimize_state;
use self::optimize_state::OptimizeState;

use crate::token::Token;
use crate::token::Token::*;
use crate::IsbfcIR;

fn _optimize(tokens: &[Token]) -> OptimizeState {
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
                    if state.sets.contains_key(&dest)
                        || state.adds.contains_key(&src)
                        || state.adds.contains_key(&dest)
                    {
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
                state
                    .tokens
                    .push(if let Some(set) = state.sets.get_mut(&offset) {
                        LoadOutSet(*set + add)
                    } else {
                        LoadOut(offset, state.adds.get(&offset).unwrap_or(&0) + add)
                    });
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

fn _optimize_loop(tokens: &[Token], outer: &mut OptimizeState) {
    let mut inner = _optimize(tokens);

    if inner.shift != 0 && inner.sets.is_empty() && inner.adds.is_empty() && inner.tokens.is_empty()
    {
        outer.tokens.push(Scan(inner.shift));
    } else if inner.shift == 0
        && inner.tokens.is_empty()
        && inner.adds.contains_key(&0)
        && inner.adds.len() == 1
    {
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
                Some(MulCopy(0, *offset, *value))
            } else {
                None
            }
        });

        if !inner.sets.is_empty() {
            let iftokens = Vec::from_iter(
                inner
                    .sets
                    .iter()
                    .map(|(offset, value)| Set(*offset, *value))
                    .chain(contents),
            );
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

impl IsbfcIR {
    /// Returns an optimized version of the intermediate representation
    pub fn optimize(&self) -> IsbfcIR {
        // Ignore sets/adds/shifts at end of file
        let mut oldtokens = _optimize(&self.tokens).tokens;
        let mut newtokens = _optimize(&oldtokens).tokens;
        while newtokens != oldtokens {
            oldtokens = newtokens;
            newtokens = _optimize(&oldtokens).tokens;
        }
        IsbfcIR { tokens: newtokens }
    }
}
