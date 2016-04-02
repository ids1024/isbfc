use std::collections::BTreeMap;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Token {
    Output,
    Input,
    Loop,
    EndLoop,
    Move(i32),
    Add(i32, i32),
    Set(i32, i32),
    MulCopy(i32, i32, i32),
    Scan(i32),
    LoadOut(i32, i32),
    LoadOutSet(i32),
    If(i32),
    EndIf,
}
use Token::*;

pub fn parse(code: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    for i in code.chars() {
        match i {
            '+' => tokens.push(Add(0, 1)),
            '-' => tokens.push(Add(0, -1)),
            '>' => tokens.push(Move(1)),
            '<' => tokens.push(Move(-1)),
            '[' => tokens.push(Loop),
            ']' => tokens.push(EndLoop),
            ',' => tokens.push(Input),
            '.' => {
                tokens.push(LoadOut(0, 0));
                tokens.push(Output);
            },
            _ => ()
        };
    }
    
    tokens
    }

pub fn optimize(tokens: Vec<Token>) -> Vec<Token> {
    let mut newtokens: Vec<Token> = Vec::with_capacity(tokens.len());
    let mut shift = 0;
    let mut do_output = false;
    // With HashMap, the order sometimes switches
    // in recursion, and the optimizer never exits.
    let mut adds: BTreeMap<i32, i32> = BTreeMap::new();
    let mut sets: BTreeMap<i32, i32> = BTreeMap::new();
    let mut pre_loop_sets: BTreeMap<i32, i32> = BTreeMap::new();

    for token in tokens.iter() {
        if *token == EndLoop && newtokens.last() == Some(&Loop) && shift == 0 && adds.contains_key(&0) {
            if adds.len() == 1 {
                newtokens.pop(); // Remove Loop
                if !sets.is_empty() {
                    newtokens.push(If(0));
                    for (offset, value) in sets.iter() {
                        newtokens.push(Set(*offset, *value));
                    }
                    sets.clear();
                    newtokens.push(Set(0, 0));
                    newtokens.push(EndIf);
                } else {
                    sets.insert(0, 0);
                }
                pre_loop_sets.clear();
                adds.clear();
                continue
            } else if adds.get(&0) == Some(&-1) {
                newtokens.pop(); // Remove Loop
                if !sets.is_empty() {
                    newtokens.push(If(0));
                    for (offset, value) in sets.iter() {
                        newtokens.push(Set(*offset, *value));
                    }
                }
                for (offset, value) in adds.iter() {
                    if *offset != 0 {
                        let src = 0;
                        let dest = *offset;
                        let mul = *value;
                        if pre_loop_sets.contains_key(&src) {
                            let val = pre_loop_sets.get(&src).unwrap() * mul;
                            newtokens.push(Add(dest, val));
                        } else {
                            newtokens.push(MulCopy(src, dest, mul));
                        }
                    }
                }
                if !sets.is_empty() {
                    newtokens.push(EndIf);
                }
                pre_loop_sets.clear();
                adds.clear();
                sets.clear();
                sets.insert(0, 0);
                continue
            }
        }

        match *token {
            Loop => {
                pre_loop_sets.clear();
                for (offset, value) in sets.iter() {
                    pre_loop_sets.insert(*offset+shift, *value);
                }
            },
            Set(_, _) | Add(_, _) | Move(_) => {},
            _ => pre_loop_sets.clear()
        }

        match *token {
            Set(_, _) | Add(_, _) | Move(_) | LoadOut(_, _) | LoadOutSet(_) | Output => {},
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
               Loop | Input | Scan(_) => {
                   newtokens.push(Move(shift));
                   shift = 0;
               },
               _ => {}
           }
        }

        match *token {
            Set(mut offset, val) => {
                offset += shift;
                // Add before Set does nothing; remove it
                adds.remove(&offset);
                sets.insert(offset, val);
            },
            Add(mut offset, mut val) => {
                offset += shift;
                if sets.contains_key(&offset) {
                    val = sets.get(&offset).unwrap() + val;
                    sets.insert(offset, val);
                } else {
                    val = adds.get(&offset).unwrap_or(&0) + val;
                    adds.insert(offset, val);
                }
            },
            MulCopy(src, dest, mul) =>
                newtokens.push(MulCopy(src+shift, dest+shift, mul)),
            // XXX Deal with shift in if, if those are ever generated
            If(offset) =>
                newtokens.push(If(offset+shift)),
            Move(offset) =>
                shift += offset,
            Output =>
                do_output = true,
            LoadOut(mut offset, add) => {
                offset += shift;
                if sets.contains_key(&offset) {
                    newtokens.push(LoadOutSet(sets.get(&offset).unwrap() + add));
                } else {
                    newtokens.push(LoadOut(offset, adds.get(&offset).unwrap_or(&0) + add));
                }
            },
            EndLoop => {
                if newtokens.last() == Some(&Loop) && shift != 0 && sets.is_empty() && adds.is_empty() {
                    newtokens.pop(); // Remove StartLoop
                    newtokens.push(Scan(shift));
                } else {
                    if shift != 0 {
                        newtokens.push(Move(shift));
                    }
                    newtokens.push(EndLoop);
                }
                shift = 0;
            },
            EndIf | LoadOutSet(_) | Loop | Input | Scan(_) =>
                newtokens.push(*token),
        }
    }

    // Any remaining add/set/shift is ignored, as it would have no effect
    if do_output {
        newtokens.push(Output);
    }

    // Optimize recursively
    if newtokens != tokens {
        optimize(newtokens)
    } else {
        newtokens
    }
}
