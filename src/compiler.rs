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

    let indent = String::from_utf8(vec![b' '; state.level * 4]).unwrap();
    /// Add line of assembly to output, with indentation and newline, using
    /// format! syntax.
    macro_rules! push_asm {
        ($fmt:expr) => {
            (writeln!(&mut state.output, concat!("{}", $fmt),
                   &indent)).unwrap()
        };
        ($fmt:expr, $($arg:tt)*) => {
            (writeln!(&mut state.output, concat!("{}", $fmt),
                   &indent,
                   $($arg)*)).unwrap()
        };
    }


    let mut outbuffpos = 0;
    for token in tokens {
        match *token {
            Add(offset, value) => {
                let dest = offset_to_operand(offset);
                if value == 1 && dest == "%r12" {
                    push_asm!("inc %r12");
                } else if value >= 1 {
                    push_asm!("addq ${}, {}", value, dest);
                } else if value == -1 && dest == "%r12" {
                    push_asm!("dec %r12");
                } else if value <= -1 {
                    push_asm!("subq ${}, {}", -value, dest);
                }
            }
            MulCopy(src_idx, dest_idx, mul) => {
                let mut src = offset_to_operand(src_idx);
                let dest = offset_to_operand(dest_idx);

                if mul != -1 && mul != 1 {
                    push_asm!("movq {}, %rax", src);
                    push_asm!("movq ${}, %rdx", mul.abs());
                    push_asm!("mulq %rdx");
                    src = "%rax".to_string();
                } else if src != "%r12" && dest != "%r12" {
                    // x86 cannot move memory to memory
                    push_asm!("movq {}, %rax", src);
                    src = "%rax".to_string();
                }

                if mul > 0 {
                    push_asm!("addq {}, {}", src, dest);
                } else {
                    push_asm!("subq {}, {}", src, dest);
                }
            }
            Set(offset, value) => {
                if offset == 0 && value == 0 {
                    push_asm!("xor %r12, %r12");
                } else {
                    push_asm!("movq ${}, {}", value, offset_to_operand(offset));
                }
            }
            Move(offset) => {
                if offset != 0 {
                    push_asm!("movq %r12, (%rbx)");
                    push_asm!("{add_sub} ${shift}, %rbx",
                              add_sub = if offset > 0 { "addq" } else { "subq" },
                              shift = offset.abs() * 8);
                    push_asm!("movq (%rbx), %r12");
                }
            }
            Loop(ref content) => {
                state.loopnum += 1;
                let curloop = state.loopnum;
                push_asm!("jmp endloop{}", curloop);
                push_asm!("loop{}:", curloop);

                compile_iter(state, &content);

                push_asm!("endloop{}:", curloop);
                push_asm!("test %r12, %r12");
                push_asm!("jnz loop{}", curloop);
            }
            If(offset, ref content) => {
                state.ifnum += 1;
                let curif = state.ifnum;
                if offset == 0 {
                    push_asm!("test %r12, %r12");
                } else {
                    push_asm!("cmpq $0, {}(%rbx)", offset * 8);
                }
                push_asm!("jz endif{}", curif);

                compile_iter(state, &content);

                push_asm!("endif{}:\n", curif);
            }
            Scan(offset) => {
                // Slighly more optimal than normal loop and move
                state.loopnum += 1;
                push_asm!("movq %r12, (%rbx)");
                push_asm!("jmp endloop{}", state.loopnum);
                push_asm!("loop{}:", state.loopnum);
                push_asm!("{add_sub} ${shift}, %rbx",
                          add_sub = if offset > 0 { "addq" } else { "subq" },
                          shift = offset.abs() * 8);
                push_asm!("endloop{}:", state.loopnum);
                push_asm!("cmp $0, (%rbx)");
                push_asm!("jnz loop{}", state.loopnum);
                push_asm!("movq (%rbx), %r12");
            }
            Input => {
                push_asm!("");

				#[cfg(target_os = "redox")]
				{
					push_asm!("movq ${}, %rax", syscall::SYS_READ);
					push_asm!("movq %rbx, %rcx");
					push_asm!("xor %rbx, %rbx");
					push_asm!("movq $1, %rdx");
					push_asm!("int $0x80");
					push_asm!("movq %rcx, %rbx");
				}

				#[cfg(not(target_os = "redox"))]
				{
					push_asm!("xor %rax, %rax");
					push_asm!("xor %rdi, %rdi");
					push_asm!("movq %rbx, %rsi");
					push_asm!("movq $1, %rdx");
					push_asm!("syscall");
				}

                push_asm!("movq (%rbx), %r12\n");
            }
            LoadOut(offset, add) => {
                let outaddr = format!("(strbuff+{})", outbuffpos);
                if offset == 0 {
                    push_asm!("movq %r12, {}", outaddr);
                } else {
                    push_asm!("movq {}(%rbx), %rax", offset * 8);
                    push_asm!("movq %rax, {}", outaddr);
                }
                if add > 0 {
                    push_asm!("addb ${}, {}", add, outaddr);
                } else if add < 0 {
                    push_asm!("subb ${}, {}", -add, outaddr);
                }
                outbuffpos += 1;
            }
            LoadOutSet(value) => {
                let outaddr = format!("(strbuff+{})", outbuffpos);
                push_asm!("movq ${}, {}", value, outaddr);
                outbuffpos += 1;
            }
            Output => {
				#[cfg(target_os = "redox")]
				{
					push_asm!("movq ${}, %rax", syscall::SYS_WRITE);
					push_asm!("movq %rbx, %r11");
					push_asm!("movq $1, %rbx");
					push_asm!("movq $strbuff, %rcx");
					push_asm!("movq ${}, %rdx", outbuffpos);
					push_asm!("int $0x80");
					push_asm!("movq %r11, %rbx\n");
				}

				#[cfg(not(target_os = "redox"))]
				{
					push_asm!("movq $1, %rax");
					push_asm!("movq $1, %rdi");
					push_asm!("movq $strbuff, %rsi");
					push_asm!("movq ${}, %rdx", outbuffpos);
					push_asm!("syscall\n");
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
        let exit = concat!("    movq $60, %rax\n",
                           "    movq $0, %rdi\n",
                           "    syscall\n");
		#[cfg(target_os = "redox")]
        let exit = format!(concat!("    movq ${}, %rax\n",
                                   "    movq $0, %rdi\n",
                                   "    int $0x80\n"),
                           syscall::SYS_EXIT);

        format!(concat!(".section .bss\n",
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
                tape_size,
                tape_size / 2,
                state.output,
                outbuffsize = state.outbuffsize,
                exit_syscall = exit)
    }
}
