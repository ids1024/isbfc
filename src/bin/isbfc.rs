use std::fs::File;
use std::io::{self, Read, Write};
use std::process::{self, Command, Stdio};

use clap::{Arg, ArgAction, ArgGroup};

use isbfc::codegen::c_codegen::{codegen, CellType};
use isbfc::{Optimizer, OPTIMIZERS};

enum Action {
    Compile,
    OutputAssembly,
    DumpAst,
    DumpIr,
    DumpLir,
}

struct Options {
    action: Action,
    output: Option<String>,
    input: String,
    tape_size: i32,
    level: u32,
    debug: bool,
    minimal_elf: bool,
    optimizer: &'static dyn Optimizer,
}

impl Options {
    fn match_options() -> Self {
        let matches = clap::Command::new("isbfc")
            .version("0.0.1")
            .author("Ian D. Scott <ian@iandouglasscott.com>")
            .about("Brainfuck compiler")
            .arg(
                Arg::new("output_asm")
                    .short('S')
                    .action(ArgAction::SetTrue)
                    .help("Assemble but do not link"),
            )
            .arg(
                Arg::new("dump_ast")
                    .long("dump-ast")
                    .action(ArgAction::SetTrue)
                    .help("Dump AST; for debugging"),
            )
            .arg(
                Arg::new("dump_ir")
                    .long("dump-ir")
                    .action(ArgAction::SetTrue)
                    .help("Dump intermediate representation; for debugging"),
            )
            .arg(
                Arg::new("dump_lir")
                    .long("dump-lir")
                    .action(ArgAction::SetTrue)
                    .help("Dump low level intermediate representation; for debugging"),
            )
            .group(ArgGroup::new("actions").args(&[
                "output_asm",
                "dump_ast",
                "dump_ir",
                "dump_lir",
            ]))
            .arg(
                Arg::new("debugging_symbols")
                    .short('g')
                    .action(ArgAction::SetTrue)
                    .help("Generate debugging information"),
            )
            .arg(
                Arg::new("out_name")
                    .short('o')
                    .help("Output file name")
                    .value_parser(clap::builder::NonEmptyStringValueParser::new())
                    .value_name("file"),
            )
            .arg(
                Arg::new("tape_size")
                    .long("tape-size")
                    .help("Size of tape")
                    .value_parser(clap::builder::NonEmptyStringValueParser::new())
                    .default_value("8192")
                    .value_name("bytes"),
            )
            .arg(
                Arg::new("minimal_elf")
                    .long("minimal-elf")
                    .action(ArgAction::SetTrue)
                    .help("Generate minimal ELF executable"),
            )
            .arg(
                Arg::new("optimizer")
                    .long("optimizer")
                    .value_parser(clap::builder::PossibleValuesParser::new(&OPTIMIZERS.keys().cloned().collect::<Vec<&str>>()))
                    .default_value("new"),
            )
            .arg(
                Arg::new("level")
                    .short('O')
                    .help("Optimization level")
                    .default_value("1"),
            )
            .arg(
                Arg::new("FILENAME")
                    .help("Source file to compile")
                    .required(true)
                    .index(1),
            )
            .get_matches();

        let action = if matches.get_flag("dump_ir") {
            Action::DumpIr
        } else if matches.get_flag("dump_ast") {
            Action::DumpAst
        } else if matches.get_flag("dump_lir") {
            Action::DumpLir
        } else if matches.get_flag("output_asm") {
            Action::OutputAssembly
        } else {
            Action::Compile
        };

        Options {
            action,
            output: matches.get_one::<String>("out_name").cloned(),
            input: matches.get_one::<String>("FILENAME").unwrap().clone(),
            tape_size: matches
                .get_one::<String>("tape_size")
                .unwrap()
                .parse::<i32>()
                .unwrap(),
            level: matches.get_one::<String>("level").unwrap().parse::<u32>().unwrap(),
            debug: matches.get_flag("debugging_symbols"),
            minimal_elf: matches.get_flag("minimal_elf"),
            optimizer: *OPTIMIZERS
                .get(matches.get_one::<String>("optimizer").unwrap().as_str())
                .unwrap(),
        }
    }

    fn get_output<'a>(&'a self, default: &'a str) -> &'a str {
        match self.output.as_ref() {
            Some(output) => output,
            None => default,
        }
    }

    fn open_output_file(&self, default: &str) -> io::Result<Box<dyn Write>> {
        let name = self.get_output(self.get_output(default));
        if name == "-" {
            Ok(Box::new(io::stdout()))
        } else {
            Ok(Box::new(File::create(&name)?))
        }
    }

    fn compile(&self, lir: Vec<isbfc::lir::LIR>) -> io::Result<String> {
        let c = codegen(&lir, CellType::U64, self.tape_size);

        let mut child = Command::new("gcc")
            .arg("-x")
            .arg("c")
            .arg("-S")
            .arg("-o")
            .arg("-") // Standard output
            .arg("-") // Standard input
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        child.stdin.take().unwrap().write_all(c.as_bytes())?;

        let mut code = String::new();
        child.stdout.take().unwrap().read_to_string(&mut code)?;

        if !child.wait()?.success() {
            process::exit(1);
        }

        Ok(code)
    }

    fn asm_and_link(&self, code: &str, name: &str, out_name: &str) {
        let o_name = format!("{}.o", name);

        println!("Assembling...");

        if isbfc::assemble(code, &o_name, self.debug).unwrap() != Some(0) {
            process::exit(1);
        }

        println!("Linking...");

        if isbfc::link(&o_name, out_name, self.minimal_elf).unwrap() != Some(0) {
            process::exit(1);
        }
    }
}

fn main() -> io::Result<()> {
    let options = Options::match_options();

    let name = options.input.rsplitn(2, '.').last().unwrap();
    let mut file = File::open(&options.input)?;
    let mut code = Vec::new();
    file.read_to_end(&mut code)?;

    let ast = match isbfc::parse(&code) {
        Ok(ast) => ast,
        Err(err) => {
            println!("Parsing error: {}", err);
            process::exit(1);
        }
    };

    let lir = options.optimizer.optimize(&ast, options.level);

    match options.action {
        Action::DumpAst => {
            let mut outfile = options.open_output_file("-")?;
            writeln!(outfile, "{:#?}", ast)?;
        }
        Action::DumpIr => {
            let mut outfile = options.open_output_file("-")?;
            options
                .optimizer
                .dumpir(&ast, options.level, &mut outfile)?;
        }
        Action::DumpLir => {
            let mut outfile = options.open_output_file("-")?;
            for i in lir {
                writeln!(outfile, "{:?}", i)?;
            }
        }
        Action::OutputAssembly => {
            println!("Compiling...");
            let output = options.compile(lir)?;
            let def_name = format!("{}.s", name);
            let mut asmfile = options.open_output_file(&def_name)?;
            asmfile.write_all(&output.into_bytes())?;
        }
        Action::Compile => {
            println!("Compiling...");
            let output = options.compile(lir)?;
            let out_name = options.get_output(name);
            options.asm_and_link(&output, &name, &out_name);
        }
    }

    Ok(())
}
