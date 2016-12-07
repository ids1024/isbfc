// NOTE: This may not work properly/at all. The compiler
// is the focus. It exist mainly for debugging: since it uses
// the same parser, it can separate parser/optimizer bugs from compiler
// bugs.

use std::env;
use std::io;
use std::io::prelude::*;
use std::fs::File;

extern crate isbfc;
use isbfc::Token;
use isbfc::Token::*;

const BUFSIZE: usize = 8192;

fn interp_iter(mem: &mut [i32; BUFSIZE],
               cur: &mut usize,
               outbuff: &mut String,
               tokens: &Vec<Token>) {
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
                io::stdin().take(1).read(&mut buffer).unwrap();
                mem[*cur] = buffer[0] as i32;
            }
            LoadOut(offset, add) => {
                outbuff.push((mem[(*cur as i32 + offset) as usize] + add) as u8 as char)
            }
            LoadOutSet(value) => outbuff.push(value as u8 as char),
            Output => {
                io::stdout().write(outbuff.as_bytes()).unwrap();
                io::stdout().flush().unwrap();
                outbuff.clear();
            }
        }
    }
}

fn main() {
    let path = env::args().nth(1).unwrap();
    let mut file = File::open(&path).unwrap();
    let mut code = String::new();
    file.read_to_string(&mut code).unwrap();

    let tokens = isbfc::parse(code.as_str()).unwrap().optimize().tokens;

    let mut mem: [i32; BUFSIZE] = [0; BUFSIZE];
    let mut cur = BUFSIZE / 2;
    let mut outbuff = String::new();

    interp_iter(&mut mem, &mut cur, &mut outbuff, &tokens);
}
