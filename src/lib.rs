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

pub fn parse(code: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    for i in code.chars() {
        match i {
            '+' => tokens.push(Token::Add(0, 1)),
            '-' => tokens.push(Token::Add(0, -1)),
            '>' => tokens.push(Token::Move(1)),
            '<' => tokens.push(Token::Move(-1)),
            '[' => tokens.push(Token::Loop),
            ']' => tokens.push(Token::EndLoop),
            ',' => tokens.push(Token::Input),
            '.' => {
                tokens.push(Token::LoadOut(0, 0));
                tokens.push(Token::Output);
            },
            _ => ()
        };
    }
    
    tokens
    }

pub fn optimize(tokens: Vec<Token>) -> Vec<Token> {
    let mut newtokens: Vec<Token> = Vec::new();
    let mut shift = 0;
    let mut do_output = false;
    let mut adds: BTreeMap<i32, i32> = BTreeMap::new();
    let mut sets: BTreeMap<i32, i32> = BTreeMap::new();
    let mut pre_loop_sets: BTreeMap<i32, i32> = BTreeMap::new();

    for token in tokens.iter() {
        let prev_token = match newtokens.last() {
            Some(tok) => Some(*tok),
            None => None
        };
        if *token == Token::EndLoop && prev_token == Some(Token::Loop) && shift == 0 && adds.contains_key(&0) {
            if adds.len() == 1 {
                newtokens.pop(); // Remove Loop
                if !sets.is_empty() {
                    newtokens.push(Token::If(0));
                    for (offset, value) in sets.iter() {
                        newtokens.push(Token::Set(*offset, *value));
                    }
                    sets.clear();
                    newtokens.push(Token::Set(0, 0));
                    newtokens.push(Token::EndIf);
                } else {
                    sets.insert(0, 0);
                }
                pre_loop_sets.clear();
                adds.clear();
                continue
            } else if adds.get(&0) == Some(&-1) {
                newtokens.pop(); // Remove Loop
                if !sets.is_empty() {
                    newtokens.push(Token::If(0));
                    for (offset, value) in sets.iter() {
                        newtokens.push(Token::Set(*offset, *value));
                    }
                }
                for (offset, value) in adds.iter() {
                    if *offset != 0 {
                        let src = 0;
                        let dest = *offset;
                        let mul = *value;
                        if pre_loop_sets.contains_key(&src) {
                            let val = pre_loop_sets.get(&src).unwrap() * mul;
                            newtokens.push(Token::Add(dest, val));
                        } else {
                            newtokens.push(Token::MulCopy(src, dest, mul));
                        }
                    }
                }
                if !sets.is_empty() {
                    newtokens.push(Token::EndIf);
                }
                pre_loop_sets.clear();
                adds.clear();
                sets.clear();
                sets.insert(0, 0);
                continue
            }
        }

        match *token {
            Token::Loop =>
                for (offset, value) in sets.iter() {
                    pre_loop_sets.insert(*offset+shift, *value);
                },
            Token::Set(_, _) | Token::Add(_, _) | Token::Move(_) => {},
            _ => pre_loop_sets.clear()
        }

        match *token {
            Token::Set(_, _) | Token::Add(_, _) | Token::Move(_) | Token::LoadOut(_, _) | Token::LoadOutSet(_) | Token::Output => {},
            _ => {
               if do_output {
                   newtokens.push(Token::Output);
                   do_output = false;
               }

               for (offset, value) in sets.iter() {
                  newtokens.push(Token::Set(*offset, *value));
               }
               for (offset, value) in adds.iter() {
                  newtokens.push(Token::Add(*offset, *value));
               }
               sets.clear();
               adds.clear();
            }
        }

        if shift != 0 {
           match *token {
               Token::Loop | Token::Input | Token::Scan(_) => {
                   newtokens.push(Token::Move(shift));
                   shift = 0;
               },
               _ => {}
           }
        }

        match *token {
            Token::Set(mut offset, val) => {
                offset += shift;
                // Add before Set does nothing; remove it
                if adds.contains_key(&offset) {
                    adds.remove(&offset);
                }
                sets.insert(offset, val);
            },
            Token::Add(mut offset, mut val) => {
                offset += shift;
                if sets.contains_key(&offset) {
                    val = sets.get(&offset).unwrap() + val;
                    sets.insert(offset, val);
                } else {
                    val = adds.get(&offset).unwrap_or(&0) + val;
                    adds.insert(offset, val);
                }
            },
            Token::MulCopy(src, dest, mul) =>
                newtokens.push(Token::MulCopy(src+shift, dest+shift, mul)),
            // XXX Deal with shift in if, if those are ever generated
            Token::If(offset) =>
                newtokens.push(Token::If(offset+shift)),
            Token::Move(offset) =>
                shift += offset,
            Token::Output =>
                do_output = true,
            Token::LoadOut(mut offset, add) => {
                offset += shift;
                if sets.contains_key(&offset) {
                    newtokens.push(Token::LoadOutSet(sets.get(&offset).unwrap() + add));
                } else {
                    newtokens.push(Token::LoadOut(offset, adds.get(&offset).unwrap_or(&0) + add));
                }
            },
            Token::EndLoop =>
                if prev_token == Some(Token::Loop) && shift != 0 && sets.is_empty() && adds.is_empty() {
                    newtokens.pop(); // Remove StartLoop
                    newtokens.push(Token::Scan(shift));
                } else {
                    if shift != 0 {
                        newtokens.push(Token::Move(shift));
                        shift = 0;
                    }
                    newtokens.push(Token::EndLoop);
                },
            Token::EndIf | Token::LoadOutSet(_) | Token::Loop | Token::Input | Token::Scan(_) =>
                newtokens.push(*token),
        }
    }

    // Any remaining add/set/shift is ignored, as it would have no effect
    if do_output {
        newtokens.push(Token::Output);
    }

    // Optimize recursively
    if newtokens != tokens {
        optimize(newtokens)
    } else {
        newtokens
    }
}
