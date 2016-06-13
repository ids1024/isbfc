// NOTE: This may not work properly/at all. The compiler
// is the focus. It exist mainly for debugging: since it uses
// the same parser, it can separate parser/optimizer bugs from compiler
// bugs.

use std::env;
use std::io;
use std::io::prelude::*;
use std::fs::File;

extern crate isbfc;
use isbfc::Token::*;
use isbfc::parse;
use isbfc::optimize;

const BUFSIZE: usize = 8192;

fn main() {
    let path = env::args().nth(1).unwrap();
    let mut file = File::open(&path).unwrap();
    let mut code = String::new();
    file.read_to_string(&mut code).unwrap();

    let tokens = parse(code.as_str());
    let tokens = optimize(tokens);

    let mut i: usize = 0;
    let mut loops: Vec<usize> = Vec::new();
    let mut mem: [i32; BUFSIZE] = [0; BUFSIZE];
    let mut cur = BUFSIZE/2;
    let mut outbuff = String::new();
    while i < tokens.len() - 1 {
        let mut token = tokens.get(i).unwrap();
        match *token {
            Add(offset, value) =>
                mem[(cur as i32 + offset) as usize] += value,
            MulCopy(src, dest, mul) =>
                mem[(cur as i32 + dest) as usize] += mem[(cur as i32 + src) as usize]*mul,
            Set(offset, value) =>
                mem[(cur as i32 + offset) as usize] = value,
            Move(offset) =>
                cur = (cur as i32 + offset) as usize,
            Loop =>
                if mem[cur] != 0 {
                    loops.push(i);
                } else {
                    let mut skiploop = 1;
                    while i < tokens.len() && skiploop > 0 {
                        i += 1;
                        token = tokens.get(i).unwrap();
                        if *token == EndLoop {
                            skiploop -= 1;
                        } else if *token == Loop {
                            skiploop += 1;
                        }
                    }
                },
            EndLoop =>
                if mem[cur] != 0 {
                    i = *loops.last().unwrap() as usize;
                } else {
                    loops.pop().unwrap();
                },
            If(offset) =>
                if mem[(cur as i32 + offset) as usize] == 0 {
                    let mut skipif = 1;
                    while i < tokens.len() && skipif > 0 {
                        i += 1;
                        token = tokens.get(i).unwrap();
                        if *token == EndIf {
                            skipif -= 1;
                        } else if let If(_) = *token {
                            skipif += 1;
                        }
                    }
                },
            EndIf => {},
            Scan(offset) =>
                while mem[cur] != 0 {
                    cur = (cur as i32 + offset) as usize;
                },
            Input => {
                let mut buffer = [0; 1];
                io::stdin().take(1).read(&mut buffer).unwrap();
                mem[cur] = buffer[0] as i32;
            },
            LoadOut(offset, add) =>
                outbuff.push((mem[(cur as i32 + offset) as usize] + add) as u8 as char),
            LoadOutSet(value) =>
                outbuff.push(value as u8 as char),
            Output => {
                io::stdout().write(outbuff.as_bytes()).unwrap();
                io::stdout().flush().unwrap();
                outbuff.clear();
            }
        }
        i += 1;
    }
}
