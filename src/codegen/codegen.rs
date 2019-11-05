// XXX: WIP, not usable

use std::fmt::{self, Write, Display, Formatter};
use std::collections::HashMap;
use crate::lir::LIR;

#[repr(u8)]
enum Reg {
    RAX,
    RCX,
    RDX,
    RBX,
    RSP,
    RBP,
    RSI,
    RDI,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15
}

impl Display for Reg {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Reg::RAX => write!(f, "%rax"),
            Reg::RCX => write!(f, "%rcx"),
            Reg::RDX => write!(f, "%rdx"),
            Reg::RBX => write!(f, "%rbx"),
            Reg::RSP => write!(f, "%rsp"),
            Reg::RBP => write!(f, "%rbp"),
            Reg::RSI => write!(f, "%rsi"),
            Reg::RDI => write!(f, "%rdi"),
            Reg::R8 => write!(f, "%r8"),
            Reg::R9 => write!(f, "%r9"),
            Reg::R10 => write!(f, "%r10"),
            Reg::R11 => write!(f, "%r11"),
            Reg::R12 => write!(f, "%r12"),
            Reg::R13 => write!(f, "%r13"),
            Reg::R14 => write!(f, "%r14"),
            Reg::R15 => write!(f, "%r15"),
        }
    }
}

enum OpSize {
    B,
    S,
    W,
    L,
    Q,
    T,
}

enum Instr {
    Movq,
    Xor,
    Int(usize),
    Syscall,
    Subb,
    Addb,
    Cmp,
    Jnz(String),
    Jz(String),
    Jmp(String),
    Test,
    Cmpq,
    Mulq,
    Dec,
    Inc,
    Label(String)
}

impl Display for Instr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Instr::Movq => Ok(()),
            Instr::Xor => Ok(()),
            Instr::Int(num) => write!(f, "int ${}", num),
            Instr::Syscall => write!(f, "syscall") ,
            Instr::Subb => Ok(()),
            Instr::Addb => Ok(()),
            Instr::Cmp => Ok(()),
            Instr::Jnz(label) => write!(f, "jnz {}", label),
            Instr::Jz(label) => write!(f, "jz {}", label),
            Instr::Jmp(label) => write!(f, "jmp {}", label),
            Instr::Test => Ok(()),
            Instr::Cmpq => Ok(()),
            Instr::Mulq => Ok(()),
            Instr::Dec => Ok(()),
            Instr::Inc => Ok(()),
            Instr::Label(label) => write!(f, "{}:", label),
        }
    }
}

// syscall(rax, rbx, rcx, rdx); clobbers %rax, %rcx, %r11
// %rdi %rsi %rdx %rcx %r8 %r9

/*
fn val_to_asm(val: impl Into<RVal>) -> String {
    match val.into() {
        Reg(reg) => {},
        Tape(offset) => {},
        Buf(buf, offset) => format!("({}+{})", buf, offset),
        Immediate(value) => format!("${}", value),
    }
}
*/

fn lir_to_instrs(lir: &[LIR], bss_bufs: &mut HashMap<String, usize>) -> Vec<Instr> {
    use LIR::*;

    let mut instrs = Vec::new();

    for i in lir {
        match i {
            Shift(shift) => {},
            Mul(dest, a, b) => {},
            Add(dest, a, b) => {
                // TODO
                /*
                if dest == a {
                } else if dest == b {
                } else {
                }
                */
            },
            Sub(dest, a, b) => {},
            Mov(dest, src) => {},
            Label(label) => instrs.push(Instr::Label(label.clone())),
            Jp(label) => instrs.push(Instr::Jmp(label.clone())),
            Jz(comparand, label) => {
                // TODO test comparand
                instrs.push(Instr::Jz(label.clone()))
            },
            Jnz(comparand, label) => {
                // TODO test comparand
                instrs.push(Instr::Jnz(label.clone()))
            },
            DeclareBssBuf(buffer, len) => { bss_bufs.insert(buffer.clone(), *len); },
            Input(buffer, offset, len) => {},
            Output(buffer, offset, len) => {},
        }
    }

    instrs
}

fn codegen(lir: &[LIR]) -> String {
    let mut output = String::new();
    let mut bss_bufs = HashMap::new();

    writeln!(output, ".section .text").unwrap();
    writeln!(output, ".global _start").unwrap();
    writeln!(output, "_start:").unwrap();

    for i in lir_to_instrs(lir, &mut bss_bufs) {
        writeln!(output, "{}", i).unwrap();
    }

    writeln!(output, ".section .bss").unwrap();
    for (name, len) in &bss_bufs {
        writeln!(output, ".lcomm {}, {}", name, len).unwrap();
    }

    output
}
