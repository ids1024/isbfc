use crate::lir::{CowStr, LVal, RVal, LIR};
use std::collections::HashMap;
use std::fmt::Write;

use cranelift::codegen::cursor::FuncCursor;
use cranelift::codegen::ir::function::Function;
use cranelift::codegen::ir::immediates::Offset32;
use cranelift::codegen::ir::InsertBuilder;
use cranelift::codegen::ir::InstBuilder;
use cranelift::prelude::*;

struct Codegen {
    cell_type: Type,
    regs: HashMap<u32, Value>,
    tape: Value,
    tape_cursor: Variable,
    bufs: HashMap<CowStr, Value>,
    labels: HashMap<CowStr, Block>,
}

impl Codegen {
    // Return value must be a pointer
    fn lval_to_cl(&self, cursor: &mut FuncCursor, val: &LVal) -> Value {
        match val {
            LVal::Reg(reg) => {}
            LVal::Tape(offset) => {}
            LVal::Buf(buf, offset) => {}
        }
        todo!()
    }

    // Return value must be an instance of cell type, not a pointer
    fn rval_to_cl(&self, cursor: &mut FuncCursor, val: &RVal) -> Value {
        match val {
            RVal::Reg(reg) => *self.regs.get(reg).unwrap(),
            // XXX offset in bytes
            // XXX offset relative to tape cursor? how to crack that?
            RVal::Tape(offset) => cursor.ins().load(
                self.cell_type,
                MemFlags::new(),
                self.tape,
                *offset * self.cell_type.bytes() as i32,
            ),
            RVal::Buf(buf, offset) => {
                let buf = *self.bufs.get(buf).unwrap();
                cursor.ins().load(
                    self.cell_type,
                    MemFlags::new(),
                    buf,
                    *offset as i32 * self.cell_type.bytes() as i32,
                )
            }
            RVal::Immediate(value) => cursor.ins().iconst(self.cell_type, i64::from(*value)),
        }
    }

    fn block(&mut self, builder: &mut FunctionBuilder, label: CowStr) -> Block {
        *self
            .labels
            .entry(label.clone())
            .or_insert_with(|| builder.create_block())
    }

    fn intr(&mut self, builder: &mut FunctionBuilder, lir: LIR) {
        match lir {
            LIR::Shift(offset) => {
                let mut tape_cursor = builder.use_var(self.tape_cursor);
                let offset = builder.cursor().ins().iconst(types::I32, i64::from(offset));
                tape_cursor = builder.cursor().ins().sadd_overflow(tape_cursor, offset).0;
                builder.def_var(self.tape_cursor, tape_cursor);
            }
            LIR::Mul(res_ptr, lhs, rhs) => {
                let res_ptr = self.lval_to_cl(&mut builder.cursor(), &res_ptr);
                let lhs = self.rval_to_cl(&mut builder.cursor(), &lhs);
                let rhs = self.rval_to_cl(&mut builder.cursor(), &rhs);
                let res = builder.cursor().ins().imul(lhs, rhs);
                builder
                    .cursor()
                    .ins()
                    .store(MemFlags::new(), res, res_ptr, Offset32::new(0));
            }
            // TODO Add, Sub
            LIR::Mov(dst, src) => {
                let dst = self.lval_to_cl(&mut builder.cursor(), &dst);
                let src = self.rval_to_cl(&mut builder.cursor(), &src);
                builder
                    .cursor()
                    .ins()
                    .store(MemFlags::new(), src, dst, Offset32::new(0));
            }
            LIR::Label(label) => {
                let block = self.block(builder, label);
                builder.switch_to_block(block);
            }
            LIR::Jp(label) => {
                let block = self.block(builder, label);
                builder.cursor().ins().jump(block, &[]);
                // XXX make sure block is terminated?
            }
            // TODO Jz, Jnz
            // TODO DeclareBssBuf
            // TODO Input, Output
            _ => {} /*
                    Shift(i32),
                    Mul(LVal, RVal, RVal),
                    Add(LVal, RVal, RVal),
                    Sub(LVal, RVal, RVal),
                    Mov(LVal, RVal),
                    Label(CowStr),
                    Jp(CowStr),
                    Jz(RVal, CowStr),
                    Jnz(RVal, CowStr),
                    DeclareBssBuf(CowStr, usize),
                    Input(CowStr, usize, usize),
                    Output(CowStr, usize, usize),
                    */
        }
    }
}

fn f() {
    let mut func = Function::new();
    let mut context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut context);
    let block = builder.create_block();
    builder.switch_to_block(block);
    InsertBuilder::new(&mut builder.cursor());
}
