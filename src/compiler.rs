use std::fmt::Write;

use crate::token::Token;
use crate::token::Token::*;
use crate::IsbfcIR;

#[cfg(target_os = "redox")]
extern crate syscall;

#[derive(Default)]
struct CompileState {
    output: String,
    loopnum: i32,
    ifnum: i32,
    outbuffsize: i32,
    level: usize,
}

/// Takes an offset from the current cell, and returns a string in assembly code
/// representing the register or memory region it is stored in
fn offset_to_operand(offset: i32) -> String {
    if offset == 0 {
        "%r12".to_string()
    } else {
        format!("{}(%rbx)", (offset * 8))
    }
}

fn compile_iter(state: &mut CompileState, tokens: &Vec<Token>) {
    state.level += 1;

    let mut outbuffpos = 0;
    for token in tokens {
        match *token {
            Add(offset, value) => {
                let dest = offset_to_operand(offset);
                if value == 1 && dest == "%r12" {
                    push_asm!(state, "inc %r12");
                } else if value >= 1 {
                    push_asm!(state, "addq ${}, {}", value, dest);
                } else if value == -1 && dest == "%r12" {
                    push_asm!(state, "dec %r12");
                } else if value <= -1 {
                    push_asm!(state, "subq ${}, {}", -value, dest);
                }
            }
            MulCopy(src_idx, dest_idx, mul) => {
                let mut src = offset_to_operand(src_idx);
                let dest = offset_to_operand(dest_idx);

                if mul != -1 && mul != 1 {
                    push_asm!(state, "movq {}, %rax", src);
                    push_asm!(state, "movq ${}, %rdx", mul.abs());
                    push_asm!(state, "mulq %rdx");
                    src = "%rax".to_string();
                } else if src != "%r12" && dest != "%r12" {
                    // x86 cannot move memory to memory
                    push_asm!(state, "movq {}, %rax", src);
                    src = "%rax".to_string();
                }

                if mul > 0 {
                    push_asm!(state, "addq {}, {}", src, dest);
                } else {
                    push_asm!(state, "subq {}, {}", src, dest);
                }
            }
            Set(offset, value) => {
                if offset == 0 && value == 0 {
                    push_asm!(state, "xor %r12, %r12");
                } else {
                    push_asm!(state, "movq ${}, {}", value, offset_to_operand(offset));
                }
            }
            Move(offset) => {
                if offset != 0 {
                    push_asm!(state, "movq %r12, (%rbx)");
                    push_asm!(
                        state,
                        "{add_sub} ${shift}, %rbx",
                        add_sub = if offset > 0 { "addq" } else { "subq" },
                        shift = offset.abs() * 8
                    );
                    push_asm!(state, "movq (%rbx), %r12");
                }
            }
            Loop(ref content) => {
                state.loopnum += 1;
                let curloop = state.loopnum;
                push_asm!(state, "jmp endloop{}", curloop);
                push_asm!(state, "loop{}:", curloop);

                compile_iter(state, &content);

                push_asm!(state, "endloop{}:", curloop);
                push_asm!(state, "test %r12, %r12");
                push_asm!(state, "jnz loop{}", curloop);
            }
            If(offset, ref content) => {
                state.ifnum += 1;
                let curif = state.ifnum;
                if offset == 0 {
                    push_asm!(state, "test %r12, %r12");
                } else {
                    push_asm!(state, "cmpq $0, {}(%rbx)", offset * 8);
                }
                push_asm!(state, "jz endif{}", curif);

                compile_iter(state, &content);

                push_asm!(state, "endif{}:\n", curif);
            }
            Scan(offset) => {
                // Slighly more optimal than normal loop and move
                state.loopnum += 1;
                push_asm!(state, "movq %r12, (%rbx)");
                push_asm!(state, "jmp endloop{}", state.loopnum);
                push_asm!(state, "loop{}:", state.loopnum);
                push_asm!(
                    state,
                    "{add_sub} ${shift}, %rbx",
                    add_sub = if offset > 0 { "addq" } else { "subq" },
                    shift = offset.abs() * 8
                );
                push_asm!(state, "endloop{}:", state.loopnum);
                push_asm!(state, "cmp $0, (%rbx)");
                push_asm!(state, "jnz loop{}", state.loopnum);
                push_asm!(state, "movq (%rbx), %r12");
            }
            Input => {
                push_asm!(state, "");

                #[cfg(target_os = "redox")]
                {
                    push_asm!(state, "movq ${}, %rax", syscall::SYS_READ);
                    push_asm!(state, "movq %rbx, %rcx");
                    push_asm!(state, "xor %rbx, %rbx");
                    push_asm!(state, "movq $1, %rdx");
                    push_asm!(state, "int $0x80");
                    push_asm!(state, "movq %rcx, %rbx");
                }

                #[cfg(not(target_os = "redox"))]
                {
                    push_asm!(state, "xor %rax, %rax");
                    push_asm!(state, "xor %rdi, %rdi");
                    push_asm!(state, "movq %rbx, %rsi");
                    push_asm!(state, "movq $1, %rdx");
                    push_asm!(state, "syscall");
                }

                push_asm!(state, "movq (%rbx), %r12\n");
            }
            LoadOut(offset, add) => {
                let outaddr = format!("(strbuff+{})", outbuffpos);
                if offset == 0 {
                    push_asm!(state, "movq %r12, {}", outaddr);
                } else {
                    push_asm!(state, "movq {}(%rbx), %rax", offset * 8);
                    push_asm!(state, "movq %rax, {}", outaddr);
                }
                if add > 0 {
                    push_asm!(state, "addb ${}, {}", add, outaddr);
                } else if add < 0 {
                    push_asm!(state, "subb ${}, {}", -add, outaddr);
                }
                outbuffpos += 1;
            }
            LoadOutSet(value) => {
                let outaddr = format!("(strbuff+{})", outbuffpos);
                push_asm!(state, "movq ${}, {}", value, outaddr);
                outbuffpos += 1;
            }
            Output => {
                #[cfg(target_os = "redox")]
                {
                    push_asm!(state, "movq ${}, %rax", syscall::SYS_WRITE);
                    push_asm!(state, "movq %rbx, %r11");
                    push_asm!(state, "movq $1, %rbx");
                    push_asm!(state, "movq $strbuff, %rcx");
                    push_asm!(state, "movq ${}, %rdx", outbuffpos);
                    push_asm!(state, "int $0x80");
                    push_asm!(state, "movq %r11, %rbx\n");
                }

                #[cfg(not(target_os = "redox"))]
                {
                    push_asm!(state, "movq $1, %rax");
                    push_asm!(state, "movq $1, %rdi");
                    push_asm!(state, "movq $strbuff, %rsi");
                    push_asm!(state, "movq ${}, %rdx", outbuffpos);
                    push_asm!(state, "syscall\n");
                }

                if state.outbuffsize < outbuffpos + 8 {
                    state.outbuffsize = outbuffpos + 8;
                }
                outbuffpos = 0;
            }
        }
    }

    state.level -= 1;
}

impl IsbfcIR {
    /// Compiles the intermediate representation to x86_64 Linux assembly
    /// returning a String
    pub fn compile(&self, tape_size: i32) -> String {
        let mut state = CompileState::default();

        compile_iter(&mut state, &self.tokens);

        // Exit syscall
        #[cfg(not(target_os = "redox"))]
        let exit = concat!(
            "    movq $60, %rax\n",
            "    movq $0, %rdi\n",
            "    syscall\n"
        );
        #[cfg(target_os = "redox")]
        let exit = format!(
            concat!(
                "    movq ${}, %rax\n",
                "    movq $0, %rdi\n",
                "    int $0x80\n"
            ),
            syscall::SYS_EXIT
        );

        format!(
            concat!(
                ".section .bss\n",
                "    .lcomm strbuff, {outbuffsize}\n",
                "    .lcomm mem, {}\n",
                "    .set startidx, mem + {}\n",
                ".section .text\n",
                ".global _start\n",
                "_start:\n",
                "    xor %r12, %r12\n",
                "    movq $startidx, %rbx\n\n",
                "{}\n",
                "{exit_syscall}\n",
            ),
            tape_size * 8,
            (tape_size / 2) * 8,
            state.output,
            outbuffsize = state.outbuffsize,
            exit_syscall = exit
        )
    }
}
