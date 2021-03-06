// NOTE: This may not work properly/at all. The compiler
// is the focus. It exist mainly for debugging: since it uses
// the same parser, it can separate parser/optimizer bugs from compiler
// bugs.

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;

extern crate isbfc;
use isbfc::{Token, IsbfcIR};
use isbfc::Token::*;

const BUFSIZE: usize = 8192;

fn interp_iter(
    mem: &mut [i32; BUFSIZE],
    cur: &mut usize,
    outbuff: &mut String,
    tokens: &[Token],
) {
    for token in tokens {
        match *token {
            Add(offset, value) => mem[(*cur as i32 + offset) as usize] += value,
            MulCopy(src, dest, mul) => {
                mem[(*cur as i32 + dest) as usize] += mem[(*cur as i32 + src) as usize] * mul
            }
            Set(offset, value) => mem[(*cur as i32 + offset) as usize] = value,
            Move(offset) => *cur = (*cur as i32 + offset) as usize,
            Loop(ref content) => {
                while mem[*cur] != 0 {
                    interp_iter(mem, cur, outbuff, content);
                }
            }
            If(offset, ref content) => {
                if mem[(*cur as i32 + offset) as usize] != 0 {
                    interp_iter(mem, cur, outbuff, content);
                }
            }
            Scan(offset) => {
                while mem[*cur] != 0 {
                    *cur = (*cur as i32 + offset) as usize;
                }
            }
            Input => {
                let mut buffer = [0; 1];
                io::stdin().take(1).read_exact(&mut buffer).unwrap();
                mem[*cur] = i32::from(buffer[0]);
            }
            LoadOut(offset, add) => {
                outbuff.push((mem[(*cur as i32 + offset) as usize] + add) as u8 as char)
            }
            LoadOutSet(value) => outbuff.push(value as u8 as char),
            Output => {
                io::stdout().write_all(outbuff.as_bytes()).unwrap();
                io::stdout().flush().unwrap();
                outbuff.clear();
            }
        }
    }
}

fn main() {
    let path = env::args().nth(1).unwrap();
    let mut file = File::open(&path).unwrap();
    let mut code = Vec::new();
    file.read_to_end(&mut code).unwrap();

    let tokens = IsbfcIR::from_ast(isbfc::parse(&code).unwrap()).optimize().tokens;

    let mut mem: [i32; BUFSIZE] = [0; BUFSIZE];
    let mut cur = BUFSIZE / 2;
    let mut outbuff = String::new();

    interp_iter(&mut mem, &mut cur, &mut outbuff, &tokens);
}
