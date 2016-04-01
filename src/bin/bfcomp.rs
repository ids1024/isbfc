use std::io::prelude::*;
use std::fs::File;
use std::process::{Command, Stdio};

extern crate clap;
use clap::{Arg, App};

extern crate isbfc;
use isbfc::Token::*;
use isbfc::parse;
use isbfc::optimize;

fn main() {
    let matches = App::new("isbfc")
        .version("0.0.1")
        .author("Ian D. Scott <ian@iandouglasscott.com>")
        .about("Brainfuck compiler")
        .arg(Arg::with_name("output_asm")
             .short("S")
             .help("Assemble but do not link"))
        .arg(Arg::with_name("out_name")
             .short("o")
             .help("Output file name")
             .takes_value(true))
        .arg(Arg::with_name("tape_size")
             .long("tape-size")
             .help("Size of tape; defaults to 8192")
             .takes_value(true)
             .empty_values(false))
        .arg(Arg::with_name("FILENAME")
             .help("Source file to compile")
             .required(true)
             .index(1))
        .get_matches();

    let mut tape_size = 8192;
    if let Some(tape_size_str) = matches.value_of("tape_size") {
        tape_size = tape_size_str.parse::<i32>().unwrap();
    }

    let path = matches.value_of("FILENAME").unwrap();
    let name = path.rsplitn(2, '.').last().unwrap();
    let out_name = matches.value_of("out_name").unwrap_or(name);
    let mut file = File::open(&path).unwrap();
    let mut code = String::new();
    file.read_to_string(&mut code).unwrap();

    println!("Compiling...");
    let output = compile(&code, tape_size);
    if matches.is_present("output_asm") {
        let mut asmfile = File::create(format!("{}.s", name)).unwrap();
        asmfile.write_all(&output.into_bytes()).unwrap();
    } else {
        asm_and_link(&output, &name, &out_name);
    }
}


fn compile(code: &str, tape_size: i32) -> String {
    let tokens = parse(code);
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
            Add(offset, value) => {
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
            MulCopy(src_idx, dest_idx, mul) => {
                let mut src = if src_idx == 0 {
                    "%r12".to_string()
                } else {
                    format!("{}(%rbx)", (src_idx*8))
                };
                let dest = if dest_idx == 0 {
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
            Set(offset, value) => {
                if offset == 0 && value == 0 {
                    output.push_str("    xor %r12, %r12\n");
                } else if offset == 0 {
                    output.push_str(&format!("    movq ${}, %r12\n", value));
                } else {
                    output.push_str(&format!("    movq ${}, {}(%rbx)\n", value, offset*8));
                }
            },
            Move(offset) => {
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
            Loop => {
                loopnum += 1;
                loops.push(loopnum);
                output.push_str(&format!(concat!(
                        "    jmp endloop{}\n",
                        "    loop{}:\n"),
                        loopnum, loopnum));
            },
            EndLoop => {
                let curloop = loops.pop().unwrap();
                output.push_str(&format!(concat!(
                            "    endloop{}:\n",
                            "    test %r12, %r12\n",
                            "    jnz loop{}\n"),
                            curloop, curloop))
            },
            If(offset) => {
                ifnum += 1;
                ifs.push(ifnum);
                if offset == 0 {
                    output.push_str("    test %r12, %r12\n");
                } else {
                    output.push_str(&format!("    cmpq $0, {}(%rbx)\n", offset*8));
                }
                output.push_str(&format!("    jz endif{}\n", ifnum));
            },
            EndIf =>
                output.push_str(&format!("    endif{}:\n", ifs.pop().unwrap())),
            Scan(offset) => {
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
            Input =>
                output.push_str(concat!("\n    xor %rax, %rax\n",
                                        "    xor %rdi, %rdi\n",
                                        "    movq %rbx, %rsi\n",
                                        "    movq $1, %rdx\n",
                                        "    syscall\n",
                                        "    movq (%rbx), %r12\n\n")),
            LoadOut(offset, add) => {
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
            LoadOutSet(value) => {
                let outaddr = format!("(strbuff+{})", outbuffpos);
                output.push_str(&format!("    movq ${}, {}\n", value, outaddr));
                outbuffpos += 1;
            },
            Output => {
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

    format!(concat!(
            ".section .bss\n",
            "    .lcomm strbuff, {outbuffsize}\n",
            "    .lcomm mem, {}\n",
            "    .set startidx, mem + {}\n",
            ".section .text\n",
            ".global _start\n",
            "_start:\n",
            "    xor %r12, %r12\n",
            "    movq $startidx, %rbx\n\n{}"),
            tape_size, tape_size/2, output, outbuffsize=outbuffsize)
}


fn asm_and_link(code: &str, name: &str, out_name: &str) {;
    println!("Assembling...");

    let mut child = Command::new("as")
        .arg("-g")
        .arg("-") // Standard input
        .arg("-o")
        .arg(format!("{}.o", name))
        .stdin(Stdio::piped())
        .spawn().unwrap();

    child.stdin.take().unwrap().write_all(code.as_bytes()).unwrap();

    let status = child.wait().unwrap();
    if status.code() == Some(0) {
        println!("Linking...");
        Command::new("ld")
            .arg(format!("{}.o", name))
            .arg("-o")
            .arg(out_name)
            .spawn().unwrap();
    }
}
