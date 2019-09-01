use std::fs::File;
use std::io::prelude::*;
use std::process::{self, Command, Stdio};
use std::os::unix::fs::PermissionsExt;

extern crate clap;
use clap::{App, Arg, ArgGroup};

extern crate isbfc;

fn main() {
    let matches = App::new("isbfc")
        .version("0.0.1")
        .author("Ian D. Scott <ian@iandouglasscott.com>")
        .about("Brainfuck compiler")
        .arg(
            Arg::with_name("output_asm")
                .short("S")
                .help("Assemble but do not link"),
        )
        .arg(
            Arg::with_name("dump_ir")
                .long("dumpir")
                .help("Dump intermediate representation; for debugging"),
        )
        .group(ArgGroup::with_name("actions").args(&["output_asm", "dump_ir"]))
        .arg(
            Arg::with_name("debugging_symbols")
                .short("g")
                .help("Generate debugging information"),
        )
        .arg(
            Arg::with_name("out_name")
                .short("o")
                .help("Output file name")
                .takes_value(true)
                .empty_values(false)
                .value_name("file"),
        )
        .arg(
            Arg::with_name("tape_size")
                .long("tape-size")
                .help("Size of tape")
                .takes_value(true)
                .empty_values(false)
                .default_value("8192")
                .value_name("bytes"),
        )
        .arg(
            Arg::with_name("minimal_elf")
                 .long("minimal-elf")
                 .help("Generate minimal ELF executable")
        )
        .arg(
            Arg::with_name("level")
                .short("O")
                .help("Optimization level")
                .takes_value(true)
                .empty_values(false)
                .default_value("1"),
        )
        .arg(
            Arg::with_name("FILENAME")
                .help("Source file to compile")
                .required(true)
                .index(1),
        )
        .get_matches();

    let tape_size = matches
        .value_of("tape_size")
        .unwrap()
        .parse::<i32>()
        .unwrap();

    let level = matches.value_of("level").unwrap().parse::<u32>().unwrap();

    let path = matches.value_of("FILENAME").unwrap();
    let name = path.rsplitn(2, '.').last().unwrap();
    let mut file = File::open(&path).unwrap();
    let mut code = Vec::new();
    file.read_to_end(&mut code).unwrap();

    let mut ir = match isbfc::parse(&code) {
        Ok(ir) => ir,
        Err(err) => {
            println!("Parsing error: {}", err);
            process::exit(1);
        }
    };
    if level > 0 {
        ir = ir.optimize();
    }

    if matches.is_present("dump_ir") {
        if let Some(out_name) = matches.value_of("out_name") {
            let mut irfile = File::create(out_name).unwrap();
            writeln!(irfile, "{:#?}", ir).unwrap();
        } else {
            println!("{:#?}", ir);
        }
    } else if matches.is_present("output_asm") {
        println!("Compiling...");
        let output = ir.compile(tape_size);
        let def_name = format!("{}.s", name);
        let out_name = matches.value_of("out_name").unwrap_or(&def_name);
        let mut asmfile = File::create(out_name).unwrap();
        asmfile.write_all(&output.into_bytes()).unwrap();
    } else {
        println!("Compiling...");
        let output = ir.compile(tape_size);
        let out_name = matches.value_of("out_name").unwrap_or(name);
        let debug = matches.is_present("debugging_symbols");
        let minimal = matches.is_present("minimal_elf");
        asm_and_link(&output, &name, &out_name, debug, minimal);
    }
}

fn object_to_binary(o_name: &str) -> (Vec<u8>, u64) {
    let mut o_file = File::open(o_name).unwrap();
    let text = isbfc::elf64_get_section(&mut o_file, b".text").unwrap().unwrap();
    let bss = isbfc::elf64_get_section(&mut o_file, b".bss").unwrap().unwrap();
    let bss_offset = (text.sh_size + 0x1000 - 1) & !(0x1000 - 1);
    let bss_size = bss.sh_size;

    let bin = Command::new("ld")
        .arg("--oformat")
        .arg("binary")
        .arg("-Ttext")
        .arg("0x401000")
        .arg("-Tbss")
        .arg(format!("0x{:x}", 0x401000 + bss_offset))
        .arg("-o")
        .arg("/dev/stdout")
        .arg(o_name)
        .output()
        .unwrap()
        .stdout;

    (bin, bss_size)
}

fn asm_and_link(code: &str, name: &str, out_name: &str, debug: bool, minimal: bool) {
    println!("Assembling...");

    let mut command = Command::new("as");
    if debug {
        command.arg("-g");
    }
    let mut child = command
        .arg("-o")
        .arg(format!("{}.o", name))
        .arg("-") // Standard input
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .take()
        .unwrap()
        .write_all(code.as_bytes())
        .unwrap();

    let status = child.wait().unwrap();
    if status.code() != Some(0) {
        process::exit(1);
    }

    println!("Linking...");

    let o_name = format!("{}.o", name);

    if minimal {
        let (bin, bss_size) = object_to_binary(&o_name);

        let mut file = File::create(out_name).unwrap();
        isbfc::elf64_write(&mut file, &bin, bss_size).unwrap();
        let mut permissions = file.metadata().unwrap().permissions();
        permissions.set_mode(permissions.mode() | 0o111);
        file.set_permissions(permissions).unwrap();
    } else {
        Command::new("ld")
            .arg("-o")
            .arg(out_name)
            .arg(o_name)
            .spawn()
            .unwrap();
    }
}
