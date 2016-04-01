use std::env;
use std::io::prelude::*;
use std::fs::File;

extern crate isbfc;
use isbfc::Token::*;
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
            Output =>
                println!("output"),
            Input =>
                println!("input"),
            Loop =>
                println!("loop"),
            EndLoop =>
                println!("endloop"),
            Move(offset) =>
                println!("move(offset={})", offset),
            Add(offset, value) =>
                println!("add(offset={}, value={})", offset, value),
            Set(offset, value) =>
                println!("set(offset={}, value={})", offset, value),
            MulCopy(src, dest, mul) =>
                println!("mulcopy(src={}, dest={}, mul={})", src, dest, mul),
            Scan(offset) =>
                println!("scan(offset={})", offset),
            LoadOut(offset, add) =>
                println!("loadout(offset={}, add={})", offset, add),
            LoadOutSet(value) =>
                println!("loadoutset(value={})", value),
            If(offset) =>
                println!("if(offset={})", offset),
            EndIf =>
                println!("endif"),
        }
    }
}
