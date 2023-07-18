use crate::lir::{CowStr, LVal, RVal, LIR};
use std::collections::HashMap;

use cranelift::prelude::*;
use cranelift_codegen::cursor::FuncCursor;
use cranelift_codegen::ir::function::Function;
use cranelift_codegen::ir::immediates::Offset32;
use cranelift_codegen::ir::InstBuilder;

struct Codegen {
    cell_type: Type,
    regs: HashMap<u32, Value>,
    tape: Value,
    tape_cursor: Variable,
    bufs: HashMap<CowStr, Value>,
    labels: HashMap<CowStr, Block>,
}

impl Codegen {
    fn new(cell_type: Type, tape: Value, tape_cursor: Variable) -> Self {
        Self {
            cell_type,
            tape,
            tape_cursor,
            regs: HashMap::new(),
            bufs: HashMap::new(),
            labels: HashMap::new(),
        }
    }

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

    fn block(&mut self, builder: &mut FunctionBuilder, label: &CowStr) -> Block {
        *self
            .labels
            .entry(label.clone())
            .or_insert_with(|| builder.create_block())
    }

    fn binary_op<F>(&self, cursor: &mut FuncCursor, res_ptr: &LVal, lhs: &RVal, rhs: &RVal, f: F)
    where
        F: Fn(&mut FuncCursor, Value, Value) -> Value,
    {
        let res_ptr = self.lval_to_cl(cursor, res_ptr);
        let lhs = self.rval_to_cl(cursor, lhs);
        let rhs = self.rval_to_cl(cursor, rhs);
        let res = f(cursor, lhs, rhs);
        cursor
            .ins()
            .store(MemFlags::new(), res, res_ptr, Offset32::new(0));
    }

    fn instr(&mut self, builder: &mut FunctionBuilder, lir: &LIR) {
        match lir {
            LIR::Shift(offset) => {
                let mut tape_cursor = builder.use_var(self.tape_cursor);
                let offset = builder
                    .cursor()
                    .ins()
                    .iconst(types::I32, i64::from(*offset));
                tape_cursor = builder.cursor().ins().sadd_overflow(tape_cursor, offset).0;
                builder.def_var(self.tape_cursor, tape_cursor);
            }
            LIR::Mul(res_ptr, lhs, rhs) => self.binary_op(
                &mut builder.cursor(),
                res_ptr,
                lhs,
                rhs,
                |cursor, lhs, rhs| cursor.ins().imul(lhs, rhs),
            ),
            LIR::Add(res_ptr, lhs, rhs) => self.binary_op(
                &mut builder.cursor(),
                res_ptr,
                lhs,
                rhs,
                |cursor, lhs, rhs| cursor.ins().iadd(lhs, rhs),
            ),
            LIR::Sub(res_ptr, lhs, rhs) => self.binary_op(
                &mut builder.cursor(),
                res_ptr,
                lhs,
                rhs,
                |cursor, lhs, rhs| cursor.ins().isub(lhs, rhs),
            ),
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
            LIR::Jz(comparand, label) => {
                // TODO
            }
            LIR::Jnz(comparand, label) => {
                // TODO
            }
            LIR::DeclareBssBuf(buffer, len) => {
                // TODO
            }
            LIR::Input(buffer, offset, len) => {
                // TODO
            }
            LIR::Output(buffer, offset, len) => {
                // TODO
            }
        }
    }
}

pub fn codegen(lir: &[LIR], cell_type: Type, tape_size: i32) -> Vec<u8> {
    let mut func = Function::new();
    let mut context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut context);
    let block = builder.create_block();
    builder.switch_to_block(block);

    // TODO
    let codegen = Codegen::new(cell_type, todo!(), todo!());

    for i in lir {
        codegen.instr(&mut builder, i);
    }

    builder.seal_all_blocks();
    builder.finalize();

    let context = cranelift_codegen::Context::for_function(func);
    let mut code = Vec::new();
    use std::str::FromStr; // TODO
    let shared_builder = cranelift_codegen::settings::builder();
    let shared_flags = cranelift_codegen::settings::Flags::new(shared_builder);
    let isa = cranelift_codegen::isa::lookup(target_lexicon::triple!("x86_64"))
        .unwrap()
        .finish(shared_flags)
        .unwrap();
    context.compile_and_emit(&*isa, &mut code, todo!()).unwrap();

    code
}
