use token::Token;
use token::Token::*;

struct CompileState {
    output: String,
    loopnum: i32,
    ifnum: i32,
    outbuffsize: i32,
}


fn compile_iter(state: &mut CompileState, tokens: Vec<Token>) {
    let mut outbuffpos = 0;
    for token in tokens {
        match token {
            Add(offset, value) => {
                let dest = if offset == 0 {
                    "%r12".to_string()
                } else {
                    format!("{}(%rbx)", (offset * 8))
                };
                if value == 1 && dest == "%r12" {
                    state.output.push_str("    inc %r12\n");
                } else if value >= 1 {
                    state.output.push_str(&format!("    addq ${}, {}\n", value, dest));
                } else if value == -1 && dest == "%r12" {
                    state.output.push_str("    dec %r12\n");
                } else if value <= -1 {
                    state.output.push_str(&format!("    subq ${}, {}\n", -value, dest));
                }
            }
            MulCopy(src_idx, dest_idx, mul) => {
                let mut src = if src_idx == 0 {
                    "%r12".to_string()
                } else {
                    format!("{}(%rbx)", (src_idx * 8))
                };
                let dest = if dest_idx == 0 {
                    "%r12".to_string()
                } else {
                    format!("{}(%rbx)", (dest_idx * 8))
                };

                if mul != -1 && mul != 1 {
                    state.output.push_str(&format!(concat!("    movq {}, %rax\n",
                                                           "    movq ${}, %rdx\n",
                                                           "    mulq %rdx\n"),
                                                   src,
                                                   mul.abs()));
                    src = "%rax".to_string();
                } else if src != "%r12" && dest != "%r12" {
                    // x86 cannot move memory to memory
                    state.output.push_str(&format!("    movq {}, %rax\n", src));
                    src = "%rax".to_string();
                }

                if mul > 0 {
                    state.output.push_str(&format!("    addq {}, {}\n", src, dest));
                } else {
                    state.output.push_str(&format!("    subq {}, {}\n", src, dest));
                }
            }
            Set(offset, value) => {
                if offset == 0 && value == 0 {
                    state.output.push_str("    xor %r12, %r12\n");
                } else if offset == 0 {
                    state.output.push_str(&format!("    movq ${}, %r12\n", value));
                } else {
                    state.output.push_str(&format!("    movq ${}, {}(%rbx)\n", value, offset * 8));
                }
            }
            Move(offset) => {
                if offset != 0 {
                    state.output.push_str("    movq %r12, (%rbx)\n");
                    if offset > 0 {
                        state.output.push_str(&format!("    addq ${}, %rbx\n", offset * 8));
                    } else {
                        state.output.push_str(&format!("    subq ${}, %rbx\n", -offset * 8));
                    }
                    state.output.push_str("    movq (%rbx), %r12\n");
                }
            }
            Loop(content) => {
                state.loopnum += 1;
                let curloop = state.loopnum;
                state.output.push_str(&format!(concat!("    jmp endloop{}\n", "    loop{}:\n"),
                                               curloop,
                                               curloop));

                compile_iter(state, content);

                state.output.push_str(&format!(concat!("    endloop{}:\n",
                                                       "    test %r12, %r12\n",
                                                       "    jnz loop{}\n"),
                                               curloop,
                                               curloop))
            }
            If(offset, content) => {
                state.ifnum += 1;
                let curif = state.ifnum;
                if offset == 0 {
                    state.output.push_str("    test %r12, %r12\n");
                } else {
                    state.output.push_str(&format!("    cmpq $0, {}(%rbx)\n", offset * 8));
                }
                state.output.push_str(&format!("    jz endif{}\n", curif));

                compile_iter(state, content);

                state.output.push_str(&format!("    endif{}:\n", curif))
            }
            Scan(offset) => {
                // Slighly more optimal than normal loop and move
                state.loopnum += 1;
                state.output.push_str(&format!(concat!("    movq %r12, (%rbx)\n",
                                                       "    jmp endloop{}\n",
                                                       "    loop{}:\n"),
                                               state.loopnum,
                                               state.loopnum));
                if offset > 0 {
                    state.output.push_str(&format!("    addq ${}, %rbx\n", offset * 8));
                } else {
                    state.output.push_str(&format!("    subq ${}, %rbx\n", -offset * 8));
                }
                state.output.push_str(&format!(concat!("    endloop{}:\n",
                                                       "    cmp $0, (%rbx)\n",
                                                       "    jnz loop{}\n",
                                                       "    movq (%rbx), %r12\n"),
                                               state.loopnum,
                                               state.loopnum));
            }
            Input => {
                state.output.push_str(concat!("\n    xor %rax, %rax\n",
                                              "    xor %rdi, %rdi\n",
                                              "    movq %rbx, %rsi\n",
                                              "    movq $1, %rdx\n",
                                              "    syscall\n",
                                              "    movq (%rbx), %r12\n\n"))
            }
            LoadOut(offset, add) => {
                let outaddr = format!("(strbuff+{})", outbuffpos);
                if offset == 0 {
                    state.output.push_str(&format!("    movq %r12, {}\n", outaddr));
                } else {
                    state.output.push_str(&format!("    movq {}(%rbx), %rax\n", offset * 8));
                    state.output.push_str(&format!("    movq %rax, {}\n", outaddr));
                }
                if add > 0 {
                    state.output.push_str(&format!("    addb ${}, {}\n", add, outaddr));
                } else if add < 0 {
                    state.output.push_str(&format!("    subb ${}, {}\n", -add, outaddr));
                }
                outbuffpos += 1;
            }
            LoadOutSet(value) => {
                let outaddr = format!("(strbuff+{})", outbuffpos);
                state.output.push_str(&format!("    movq ${}, {}\n", value, outaddr));
                outbuffpos += 1;
            }
            Output => {
                state.output.push_str(&format!(concat!("    movq $1, %rax\n",
                                                       "    movq $1, %rdi\n",
                                                       "    movq $strbuff, %rsi\n",
                                                       "    movq ${}, %rdx\n",
                                                       "    syscall\n\n"),
                                               outbuffpos));

                if state.outbuffsize < outbuffpos + 8 {
                    state.outbuffsize = outbuffpos + 8;
                }
                outbuffpos = 0;
            }
        }
    }
}

pub fn compile(tokens: Vec<Token>, tape_size: i32) -> String {
    let mut state = CompileState {
        output: String::new(),
        loopnum: 0,
        ifnum: 0,
        outbuffsize: 0,
    };

    compile_iter(&mut state, tokens);

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
                    // Exit syscall
                    "    movq $60, %rax\n",
                    "    movq $0, %rdi\n",
                    "    syscall\n"),
            tape_size,
            tape_size / 2,
            state.output,
            outbuffsize = state.outbuffsize)
}
