// Trivial "optimizer" that turns AST to LIR directly.
// Serves as an example optimizer implementation, and perhaps
// useful as a reference for benchmarking and debugging.

use std::io::Write;
use crate::{AST, LIR, LIRBuilder};
use super::Optimizer;

pub struct SimpleOptimizer;

impl Optimizer for SimpleOptimizer {
    fn optimize(ast: &[AST], level: u32) -> Vec<LIR> {
        let mut loopnum = 0;
        let mut lir = LIRBuilder::new();
        optimize(ast, level, &mut loopnum, &mut lir);
        lir.build()
    }

    fn dumpir(ast: &[AST], level: u32, file: &mut impl Write) -> std::io::Result<(())> {
        // Optimizer lacks its own IR, so dump LIR
        writeln!(file, "{:#?}", Self::optimize(ast, level))
    }
}

fn optimize(ast: &[AST], level: u32, loopnum: &mut u32, lir: &mut LIRBuilder) {
    use crate::lir::prelude::*;

    lir.declare_bss_buf("strbuf".to_string(), 1);

    for i in ast {
        match i {
            AST::Output => {
                lir.mov(Buf("strbuf".to_string(), 0), Tape(0));
                lir.output("strbuf".to_string(), 0, 1);
            },
            AST::Input => {
                lir.input("strbuf".to_string(), 0, 1);
                lir.mov(Tape(0), Buf("strbuf".to_string(), 0));
            }
            AST::Loop(ast) => {
                *loopnum += 1;
                let startlabel = format!("loop{}", loopnum);
                let endlabel = format!("endloop{}", loopnum);

                lir.jp(endlabel.clone());
                lir.label(startlabel.clone());

                optimize(ast, level, loopnum, lir);

                lir.label(endlabel.clone());
                lir.jnz(Tape(0), startlabel.clone());


            }
            AST::Right => { lir.shift(1); },
            AST::Left => { lir.shift(-1); },
            AST::Inc => { lir.add(Reg(0), Reg(0), Immediate(1)); },
            AST::Dec => { lir.add(Reg(0), Reg(0), Immediate(-1)); },
        }
    }
}
