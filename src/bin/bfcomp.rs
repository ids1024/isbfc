use std::env;
use std::io::prelude::*;
use std::fs::File;
use std::process::Command;

extern crate isbfc;
use isbfc::Token;
use isbfc::parse;
use isbfc::optimize;

const BUFSIZE: i32 = 8192;

fn main() {
    let path = env::args().nth(1).unwrap();
    let mut file = File::open(&path).unwrap();
    let mut code = String::new();
    file.read_to_string(&mut code).unwrap();

    println!("Compiling...");
 
    let tokens = parse(code.as_str());
    let tokens = optimize(tokens);

    let mut output = String::new();
    let mut loops: Vec<i32> = Vec::new();
    let mut ifs: Vec<i32> = Vec::new();
    let mut loopnum = 0;
    let mut ifnum = 0;
    let mut outbuffpos = 0;
    let mut outbuffsize = 0;
    for token in tokens.iter() {
        match *token {
            Token::Add(offset, value) => {
                let dest = if offset == 0 {
                    "%r12".to_string()
                } else {
                    format!("{}(%rbx)", (offset*8))
                };
                if value == 1 && dest == "%r12" {
                    output.push_str("    inc %r12\n");
                } else if value >= 1 {
                    output.push_str(&format!("    addq ${}, {}\n", value, dest));
                } else if value == -1 && dest == "%r12" {
                    output.push_str("    dec %r12\n");
                } else if value <= -1 {
                    output.push_str(&format!("    subq ${}, {}\n", -value, dest));
                }
            },
            Token::MulCopy(src_idx, dest_idx, mul) => {
                let mut src = if src_idx == 0 {
                    "%r12".to_string()
                } else {
                    format!("{}(%rbx)", (src_idx*8))
                };
                let dest = if src_idx == 0 {
                    "%r12".to_string()
                } else {
                    format!("{}(%rbx)", (dest_idx*8))
                };

                if mul != -1 && mul != 1 {
                    output.push_str(&format!(concat!(
                                "    movq {}, %rax\n",
                                "    movq ${}, %rdx\n",
                                "    mulq %rdx\n"), src, mul.abs()));
                    src = "%rax".to_string();
                } else if src != "%r12" && dest != "%r12" {
                    // x86 cannot move memory to memory
                    output.push_str(&format!("    movq {}, %rax\n", src));
                    src = "%rax".to_string();
                }

                if mul > 0 {
                    output.push_str(&format!("    addq {}, {}\n", src, dest));
                } else {
                    output.push_str(&format!("    subq {}, {}\n", src, dest));
                }
            },
            Token::Set(offset, value) => {
                if offset == 0 && value == 0 {
                    output.push_str("    xor %r12, %r12\n");
                } else if offset == 0 {
                    output.push_str(&format!("    movq ${}, %r12\n", value));
                } else {
                    output.push_str(&format!("    movq ${}, {}(%rbx)\n", value, offset*8));
                }
            },
            Token::Move(offset) => {
                if offset != 0 {
                    output.push_str("    movq %r12, (%rbx)\n");
                    if offset > 0 {
                        output.push_str(&format!("    addq ${}, %rbx\n", offset*8));
                    } else {
                        output.push_str(&format!("    subq ${}, %rbx\n", -offset*8));
                    }
                    output.push_str("    movq (%rbx), %r12\n");
                }
            },
            Token::Loop => {
                loopnum += 1;
                loops.push(loopnum);
                output.push_str(&format!(concat!(
                        "    jmp endloop{}\n",
                        "    loop{}:\n"),
                        loopnum, loopnum));
            },
            Token::EndLoop => {
                let curloop = loops.pop().unwrap();
                output.push_str(&format!(concat!(
                            "    endloop{}:\n",
                            "    test %r12, %r12\n",
                            "    jnz loop{}\n"),
                            curloop, curloop))
            },
            Token::If(offset) => {
                ifnum += 1;
                ifs.push(ifnum);
                if offset == 0 {
                    output.push_str("    test %r12, %r12\n");
                } else {
                    output.push_str(&format!("    cmpq $0, {}(%rbx)\n", offset*8));
                }
                output.push_str(&format!("    jz endif{}\n", ifnum));
            },
            Token::EndIf =>
                output.push_str(&format!("    endif{}:\n", ifs.pop().unwrap())),
            Token::Scan(offset) => {
                // Slighly more optimal than normal loop and move
                loopnum += 1;
                output.push_str(&format!(concat!(
                            "    movq %r12, (%rbx)\n",
                            "    jmp endloop{}\n",
                            "    loop{}:\n"),
                            loopnum, loopnum));
                if offset > 0 {
                    output.push_str(&format!("    addq ${}, %rbx\n", offset*8));
                } else {
                    output.push_str(&format!("    subq ${}, %rbx\n", -offset*8));
                }
                output.push_str(&format!(concat!(
                            "    endloop{}:\n",
                            "    cmp $0, (%rbx)\n",
                            "    jnz loop{}\n",
                            "    movq (%rbx), %r12\n"),
                            loopnum, loopnum));
            },
            Token::Input =>
                output.push_str(concat!("\n    xor %rax, %rax\n",
                                        "    xor %rdi, %rdi\n",
                                        "    movq %rbx, %rsi\n",
                                        "    movq $1, %rdx\n",
                                        "    syscall\n",
                                        "    movq (%rbx), %r12\n\n")),
            Token::LoadOut(offset, add) => {
                let outaddr = format!("(strbuff+{})", outbuffpos);
                if offset == 0 {
                    output.push_str(&format!("    movq %r12, {}\n", outaddr));
                } else {
                    output.push_str(&format!("    movq {}(%rbx), %rax\n", offset*8));
                    output.push_str(&format!("    movq %rax, {}\n", outaddr));
                }
                if add > 0 {
                    output.push_str(&format!("    addb ${}, {}\n", add, outaddr));
                } else if add < 0 {
                    output.push_str(&format!("    subb ${}, {}\n", -add, outaddr));
                }
                outbuffpos += 1;
            },
            Token::LoadOutSet(value) => {
                let outaddr = format!("(strbuff+{})", outbuffpos);
                output.push_str(&format!("    movq ${}, {}\n", value, outaddr));
                outbuffpos += 1;
            },
            Token::Output => {
                output.push_str(&format!(concat!(
                            "    movq $1, %rax\n",
                            "    movq $1, %rdi\n",
                            "    movq $strbuff, %rsi\n",
                            "    movq ${}, %rdx\n",
                            "    syscall\n\n"),
                            outbuffpos));

                if outbuffsize < outbuffpos + 8 {
                    outbuffsize = outbuffpos + 8;
                }
                outbuffpos = 0;
            }
        }
    }

    // Exit syscall
    output.push_str(concat!("\n    movq $60, %rax\n",
                            "    movq $0, %rdi\n",
                            "    syscall\n"));

    output = format!(concat!(
            ".section .bss\n",
            "    .lcomm strbuff, {outbuffsize}\n",
            "    .lcomm mem, {}\n",
            "    .set startidx, mem + {}\n",
            ".section .text\n",
            ".global _start\n",
            "_start:\n",
            "    xor %r12, %r12\n",
            "    movq $startidx, %rbx\n\n{}"),
            BUFSIZE, BUFSIZE/2, output, outbuffsize=outbuffsize);

    let name = path.rsplitn(2, '.').last().unwrap();
    let mut asmfile = File::create(format!("{}.s", name)).unwrap();
    asmfile.write_all(&output.into_bytes()).unwrap();

    println!("Assembling...");
    let status = Command::new("as")
        .arg("-g")
        .arg(format!("{}.s", name))
        .arg("-o")
        .arg(format!("{}.o", name))
        .status().unwrap();
    if status.code() == Some(0) {
        println!("Linking...");
        Command::new("ld")
            .arg(format!("{}.o", name))
            .arg("-o")
            .arg(name)
            .spawn().unwrap();
    }
}