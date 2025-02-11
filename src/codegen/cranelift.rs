use crate::lir::{CowStr, LVal, RVal, LIR};
use std::collections::HashMap;

use cranelift::prelude::*;
use cranelift_codegen::cursor::FuncCursor;
use cranelift_codegen::ir::function::Function;
use cranelift_codegen::ir::immediates::Offset32;
use cranelift_codegen::ir::InstBuilder;
use cranelift_control::ControlPlane;

struct Codegen {
    cell_type: Type,
    regs: HashMap<u32, Variable>,
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

    fn store(&mut self, builder: &mut FunctionBuilder, lval: &LVal, val: Value) {
        match lval {
            LVal::Reg(reg) => {
                let var = *self.regs.entry(*reg).or_insert_with(|| {
                    let var = Variable::from_u32(reg + 1); // TODO?
                    builder.declare_var(var, self.cell_type);
                    var
                });
                builder.def_var(var, val);
            }
            LVal::Tape(offset) => {
                builder.ins().store(
                    MemFlags::new(),
                    val,
                    self.tape,
                    *offset * self.cell_type.bytes() as i32,
                );
            }
            LVal::Buf(buf, offset) => {
                let buf = *self.bufs.entry(buf.clone()).or_insert_with(|| {
                    // XXX
                    builder.ins().get_stack_pointer(self.cell_type)
                });
                builder.ins().store(
                    MemFlags::new(),
                    val,
                    buf,
                    *offset as i32 * self.cell_type.bytes() as i32,
                );
            }
        }
    }

    // Return value must be an instance of cell type, not a pointer
    fn rval_to_cl(&self, builder: &mut FunctionBuilder, val: &RVal) -> Value {
        match val {
            RVal::Reg(reg) => builder.use_var(*self.regs.get(reg).unwrap()),
            // XXX offset in bytes
            // XXX offset relative to tape cursor? how to crack that?
            RVal::Tape(offset) => builder.ins().load(
                self.cell_type,
                MemFlags::new(),
                self.tape,
                *offset * self.cell_type.bytes() as i32,
            ),
            RVal::Buf(buf, offset) => {
                let buf = *self.bufs.get(buf).unwrap();
                builder.ins().load(
                    self.cell_type,
                    MemFlags::new(),
                    buf,
                    *offset as i32 * self.cell_type.bytes() as i32,
                )
            }
            RVal::Immediate(value) => builder.ins().iconst(self.cell_type, i64::from(*value)),
        }
    }

    fn block(&mut self, builder: &mut FunctionBuilder, label: &CowStr) -> Block {
        *self
            .labels
            .entry(label.clone())
            .or_insert_with(|| builder.create_block())
    }

    fn binary_op<F>(
        &mut self,
        builder: &mut FunctionBuilder,
        lval: &LVal,
        lhs: &RVal,
        rhs: &RVal,
        f: F,
    ) where
        F: Fn(&mut FuncCursor, Value, Value) -> Value,
    {
        let lhs = self.rval_to_cl(builder, lhs);
        let rhs = self.rval_to_cl(builder, rhs);
        let res = f(&mut builder.cursor(), lhs, rhs);
        self.store(builder, lval, res);
    }

    fn instr(&mut self, builder: &mut FunctionBuilder, lir: &LIR) {
        match lir {
            LIR::Shift(offset) => {
                let mut tape_cursor = builder.use_var(self.tape_cursor);
                let offset = builder.ins().iconst(types::I32, i64::from(*offset));
                tape_cursor = builder.ins().sadd_overflow(tape_cursor, offset).0;
                builder.def_var(self.tape_cursor, tape_cursor);
            }
            LIR::Mul(res_ptr, lhs, rhs) => {
                self.binary_op(builder, res_ptr, lhs, rhs, |cursor, lhs, rhs| {
                    cursor.ins().imul(lhs, rhs)
                })
            }
            LIR::Add(res_ptr, lhs, rhs) => {
                self.binary_op(builder, res_ptr, lhs, rhs, |cursor, lhs, rhs| {
                    cursor.ins().iadd(lhs, rhs)
                })
            }
            LIR::Sub(res_ptr, lhs, rhs) => {
                self.binary_op(builder, res_ptr, lhs, rhs, |cursor, lhs, rhs| {
                    cursor.ins().isub(lhs, rhs)
                })
            }
            LIR::Mov(dst, src) => {
                let src = self.rval_to_cl(builder, &src);
                self.store(builder, dst, src);
            }
            LIR::Label(label) => {
                // TODO seal current block
                let block = self.block(builder, label);
                // Jump to next block; XXX if not ended on jump?
                builder.ins().jump(block, &[]);
                builder.switch_to_block(block);
            }
            LIR::Jp(label) => {
                let block = self.block(builder, label);
                builder.ins().jump(block, &[]);
                // XXX make sure block is terminated?

                // XXX
                let next_block = builder.create_block();
                builder.switch_to_block(next_block);
            }
            LIR::Jz(comparand, label) => {
                let block = self.block(builder, label);
                let else_block = builder.create_block(); // XXX continue? Add block?
                let value = self.rval_to_cl(builder, &comparand);
                let value = builder.ins().bnot(value);
                builder.ins().brif(value, block, &[], else_block, &[]);
                builder.switch_to_block(else_block);
            }
            LIR::Jnz(comparand, label) => {
                let block = self.block(builder, label);
                let else_block = builder.create_block(); // XXX continue? Add block?
                let value = self.rval_to_cl(builder, &comparand);
                builder.ins().brif(value, block, &[], else_block, &[]);
                builder.switch_to_block(else_block);
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

pub fn codegen_fn(lir: &[LIR], cell_type: Type, tape_size: i32) -> Function {
    let mut func = Function::new();
    let mut context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut context);
    let block = builder.create_block();
    builder.switch_to_block(block);

    // XXX
    let tape_ptr = builder.ins().get_stack_pointer(cell_type);

    let tape_var = Variable::from_u32(0); // XXX?
    builder.declare_var(tape_var, cell_type);

    // TODO
    let mut codegen = Codegen::new(cell_type, tape_ptr, tape_var);

    for i in lir {
        codegen.instr(&mut builder, i);
    }

    builder.seal_all_blocks();
    builder.finalize();

    func
}

pub fn codegen(lir: &[LIR], cell_type: Type, tape_size: i32) -> Vec<u8> {
    let func = codegen_fn(lir, cell_type, tape_size);
    let mut context = cranelift_codegen::Context::for_function(func);
    let shared_builder = cranelift_codegen::settings::builder();
    let shared_flags = cranelift_codegen::settings::Flags::new(shared_builder);
    let isa = cranelift_codegen::isa::lookup(target_lexicon::triple!("x86_64"))
        .unwrap()
        .finish(shared_flags)
        .unwrap();
    let compiled_code = context
        .compile(&*isa, &mut ControlPlane::default())
        .unwrap();

    compiled_code.buffer.data().to_vec()
}
