use std::fs::File;
use std::io::{Read, Write};
use std::process::{self, Command, Stdio};

use clap::{App, Arg, ArgGroup};

use isbfc::codegen::c_codegen::{codegen, CellType};
use isbfc::OPTIMIZERS;

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
                .long("dump-ir")
                .help("Dump intermediate representation; for debugging"),
        )
        .arg(
            Arg::with_name("dump_lir")
                .long("dump-lir")
                .help("Dump low level intermediate representation; for debugging"),
        )
        .group(ArgGroup::with_name("actions").args(&["output_asm", "dump_ir", "dump_lir"]))
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
                .help("Generate minimal ELF executable"),
        )
        .arg(
            Arg::with_name("optimizer")
                .long("optimizer")
                .takes_value(true)
                .possible_values(&OPTIMIZERS.keys().cloned().collect::<Vec<&str>>())
                .default_value("old"),
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

    let ast = match isbfc::parse(&code) {
        Ok(ast) => ast,
        Err(err) => {
            println!("Parsing error: {}", err);
            process::exit(1);
        }
    };

    let optimizer = OPTIMIZERS
        .get(matches.value_of("optimizer").unwrap())
        .unwrap();

    let lir = optimizer.optimize(&ast, level);

    if matches.is_present("dump_ir") {
        if let Some(out_name) = matches.value_of("out_name") {
            let mut irfile = File::create(out_name).unwrap();
            optimizer.dumpir(&ast, level, &mut irfile).unwrap();
        } else {
            optimizer
                .dumpir(&ast, level, &mut std::io::stdout())
                .unwrap();
        };
    } else if matches.is_present("dump_lir") {
        if let Some(out_name) = matches.value_of("out_name") {
            let mut irfile = File::create(out_name).unwrap();
            writeln!(irfile, "{:#?}", lir).unwrap();
        } else {
            println!("{:#?}", lir);
        };
    } else if matches.is_present("output_asm") {
        println!("Compiling...");
        let output = compile(lir, tape_size);
        let def_name = format!("{}.s", name);
        let out_name = matches.value_of("out_name").unwrap_or(&def_name);
        let mut asmfile = File::create(out_name).unwrap();
        asmfile.write_all(&output.into_bytes()).unwrap();
    } else {
        println!("Compiling...");
        let output = compile(lir, tape_size);
        let out_name = matches.value_of("out_name").unwrap_or(name);
        let debug = matches.is_present("debugging_symbols");
        let minimal = matches.is_present("minimal_elf");
        asm_and_link(&output, &name, &out_name, debug, minimal);
    }
}

pub fn compile(lir: Vec<isbfc::lir::LIR>, tape_size: i32) -> String {
    // TODO: avoid unwrap

    let c = codegen(&lir, CellType::U64, tape_size);

    let mut child = Command::new("gcc")
        .arg("-x")
        .arg("c")
        .arg("-S")
        .arg("-o")
        .arg("-") // Standard output
        .arg("-") // Standard input
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child.stdin.take().unwrap().write_all(c.as_bytes()).unwrap();

    let mut code = String::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut code)
        .unwrap();

    if !child.wait().unwrap().success() {
        process::exit(1);
    }

    code
}

fn asm_and_link(code: &str, name: &str, out_name: &str, debug: bool, minimal: bool) {
    let o_name = format!("{}.o", name);

    println!("Assembling...");

    if isbfc::assemble(code, &o_name, debug).unwrap() != Some(0) {
        process::exit(1);
    }

    println!("Linking...");

    if isbfc::link(&o_name, out_name, minimal).unwrap() != Some(0) {
        process::exit(1);
    }
}
