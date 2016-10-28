use std::fmt;
use token::Token;

pub struct IsbfcIR {
    pub tokens: Vec<Token>
}

impl fmt::Debug for IsbfcIR {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.tokens.fmt(f)
    }
}
