use std::fmt::Write;
use std::collections::HashMap;
use crate::lir::{LIR, LVal, RVal};
use LIR::*;

fn lval_to_c(val: &LVal) -> String {
    match val {
        LVal::Reg(reg) => format!("uint64_t r{}", reg),
        LVal::Tape(offset) => format!("tape[cursor + {}]", offset),
        LVal::Buf(buf, offset) => format!("{}[{}]", buf, offset),
    }
}

fn rval_to_c(val: &RVal) -> String {
    match val {
        RVal::Reg(reg) => format!("r{}", reg),
        RVal::Tape(offset) => format!("tape[cursor + {}]", offset),
        RVal::Buf(buf, offset) => format!("{}[{}]", buf, offset),
        RVal::Immediate(value) => format!("{}", value),
    }
}

pub fn codegen(lir: &[LIR]) -> String {
    let mut output = String::new();

    macro_rules! push_asm {
        ($($arg:tt)*) => {
            (writeln!(output, $($arg)*)).unwrap()
        };
    }

    let mut bss_bufs = HashMap::new();

    for i in lir {
        match i {
            Shift(shift) => push_asm!("cursor += {};", shift),
            Mul(dest, a, b) => push_asm!("{} = {} * {};", lval_to_c(dest), rval_to_c(a), rval_to_c(b)),
            Add(dest, a, b) => push_asm!("{} = {} + {};", lval_to_c(dest), rval_to_c(a), rval_to_c(b)),
            Sub(dest, a, b) => push_asm!("{} = {} - {};", lval_to_c(dest), rval_to_c(a), rval_to_c(b)),
            Mov(dest, src) => push_asm!("{} = {};", lval_to_c(dest), rval_to_c(src)),
            // https://stackoverflow.com/questions/18496282/why-do-i-get-a-label-can-only-be-part-of-a-statement-and-a-declaration-is-not-a
            Label(label) => push_asm!("{}: ;", label),
            Jp(label) => push_asm!("goto {};", label),
            Jz(comparand, label) => push_asm!("if ({} == 0) {{ goto {}; }}", rval_to_c(comparand), label),
            Jnz(comparand, label) => push_asm!("if ({} != 0) {{ goto {}; }}", rval_to_c(comparand), label),
            DeclareBssBuf(buffer, len) => { bss_bufs.insert(buffer, len); },
            Input(buffer, offset, len) => {}, // XXX
            Output(buffer, offset, len) => push_asm!("fwrite({}+{}, 1, {}, stdout);", buffer, offset, len),
        }
    }

    let mut bss = String::new();
    for (name, len) in bss_bufs {
        writeln!(bss, "char {}[{}];", name, len).unwrap();
    }

    return format!(concat!(
        "#include <stdint.h>\n",
        "#include <stdio.h>\n",
        "uint64_t tape[8192];\n",
        "size_t cursor = 4096;\n",
        "{}\n",
        "int main() {{\n",
        "{}\n",
        "}}\n"), bss, output)
}
