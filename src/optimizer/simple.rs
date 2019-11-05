// Trivial "optimizer" that turns AST to LIR directly.
// Serves as an example optimizer implementation, and perhaps
// useful as a reference for benchmarking and debugging.

use std::io::Write;
use crate::{AST, LIR};
use super::Optimizer;

pub struct SimpleOptimizer;

impl Optimizer for SimpleOptimizer {
    fn optimize(ast: &[AST], level: u32) -> Vec<LIR> {
        let mut loopnum = 0;
        optimize(ast, level, &mut loopnum)
    }

    fn dumpir(ast: &[AST], level: u32, file: &mut impl Write) -> std::io::Result<(())> {
        // Optimizer lacks its own IR, so dump LIR
        writeln!(file, "{:#?}", Self::optimize(ast, level))
    }
}

fn optimize(ast: &[AST], level: u32, loopnum: &mut u32) -> Vec<LIR> {
    use crate::lir::lir::*;

    let mut lir = Vec::new();

    lir.push(declare_bss_buf("strbuf".to_string(), 1));

    for i in ast {
        match i {
            AST::Output => {
                lir.push(mov(Buf("strbuf".to_string(), 0), Tape(0)));
                lir.push(output("strbuf".to_string(), 0, 1));
            },
            AST::Input => {
                lir.push(input("strbuf".to_string(), 0, 1));
                lir.push(mov(Tape(0), Buf("strbuf".to_string(), 0)));
            }
            AST::Loop(ast) => {
                *loopnum += 1;
                let startlabel = format!("loop{}", loopnum);
                let endlabel = format!("endloop{}", loopnum);

                lir.push(jp(endlabel.clone()));
                lir.push(label(startlabel.clone()));

                lir.extend(optimize(ast, level, loopnum));

                lir.push(label(endlabel.clone()));
                lir.push(jnz(Tape(0), startlabel.clone()));


            }
            AST::Right => lir.push(shift(1)),
            AST::Left => lir.push(shift(-1)),
            AST::Inc => lir.push(add(Reg(0), Reg(0), Immediate(1))),
            AST::Dec => lir.push(add(Reg(0), Reg(0), Immediate(-1))),
        }
    }
    lir
}
