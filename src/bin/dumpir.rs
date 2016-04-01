use std::env;
use std::io::prelude::*;
use std::fs::File;

extern crate isbfc;
use isbfc::Token;
use isbfc::parse;
use isbfc::optimize;

fn main() {
    let path = env::args().nth(1).unwrap();
    let mut file = File::open(path).unwrap();
    let mut code = String::new();
    file.read_to_string(&mut code).unwrap();
    
    let tokens = parse(&code);
    let tokens = optimize(tokens);

    for token in tokens.iter() {
        match *token {
            Token::Output =>
                println!("output"),
            Token::Input =>
                println!("input"),
            Token::Loop =>
                println!("loop"),
            Token::EndLoop =>
                println!("endloop"),
            Token::Move(offset) =>
                println!("move(offset={})", offset),
            Token::Add(offset, value) =>
                println!("add(offset={}, value={})", offset, value),
            Token::Set(offset, value) =>
                println!("set(offset={}, value={})", offset, value),
            Token::MulCopy(src, dest, mul) =>
                println!("mulcopy(src={}, dest={}, mul={})", src, dest, mul),
            Token::Scan(offset) =>
                println!("scan(offset={})", offset),
            Token::LoadOut(offset, add) =>
                println!("loadout(offset={}, add={})", offset, add),
            Token::LoadOutSet(value) =>
                println!("loadoutset(value={})", value),
            Token::If(offset) =>
                println!("if(offset={})", offset),
            Token::EndIf =>
                println!("endif"),
        }
    }
}
