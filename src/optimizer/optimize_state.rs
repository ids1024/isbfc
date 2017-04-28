use std::collections::BTreeMap;
use token::Token;

#[derive(Default)]
pub struct OptimizeState {
    pub tokens: Vec<Token>,
    // With HashMap, the order sometimes switches
    // in recursion, and the optimizer never exits.
    pub adds: BTreeMap<i32, i32>,
    pub sets: BTreeMap<i32, i32>,
    pub shift: i32,
}

impl OptimizeState {
    pub fn apply_shift(&mut self) {
        if self.shift != 0 {
            self.tokens.push(Token::Move(self.shift));
            self.shift = 0;
        }
    }

    pub fn apply_adds_sets(&mut self) {
        for (offset, value) in &self.sets {
            self.tokens.push(Token::Set(*offset, *value));
        }
        for (offset, value) in &self.adds {
            self.tokens.push(Token::Add(*offset, *value));
        }
        self.sets.clear();
        self.adds.clear();
    }

    pub fn add(&mut self, offset: i32, mut value: i32) {
        if let Some(set) = self.sets.get_mut(&offset) {
            *set += value;
        } else {
            value = self.adds.get(&offset).unwrap_or(&0) + value;
            if value != 0 {
                self.adds.insert(offset, value);
            } else {
                self.adds.remove(&offset);
            }
        }
    }

    pub fn set(&mut self, offset: i32, value: i32) {
        // Add before Set does nothing; remove it
        self.adds.remove(&offset);
        self.sets.insert(offset, value);
    }
}
