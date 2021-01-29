#[macro_use]
extern crate lalrpop_util;

use crate::ast::Program;
use lalrpop_util::ParseError;
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use std::{fs, io};
use structopt::StructOpt;

lalrpop_mod!(
    #[allow(clippy::all)]
    parser
);

mod ast;
mod ir;
mod lower_ast;
mod lower_ir;

#[derive(StructOpt)]
struct CliArgs {
    #[structopt(parse(from_os_str))]
    input_file: PathBuf,

    #[structopt(
        parse(from_os_str),
        long = "output",
        short = "o",
        default_value = "out.obj"
    )]
    output_file: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Setup logging and parse CLI args
    env_logger::init();
    let args = CliArgs::from_args();

    // Load the assembly file
    let file_str = fs::read_to_string(&args.input_file)?;
    let program = parse(&file_str);

    assemble_file(program, args)?;

    Ok(())
}

fn assemble_file(program: Program, args: CliArgs) -> io::Result<()> {
    println!("{:#?}", program);
    let program_ir = program.lower();
    println!("{:#?}", program_ir);
    let program_mips = program_ir.lower();
    println!("{:#x?}", program_mips);

    let mut output = File::create(&args.output_file)?;
    program_mips.write(&mut output)
}

/// Parse the MIPS program. If there are errors during parsing, the program will
/// exit.
fn parse(file_str: &str) -> Program {
    match parser::ProgramParser::new().parse(file_str) {
        Ok(parsed_ast) => parsed_ast,
        Err(ParseError::InvalidToken { location }) => {
            let (line, col) = index_to_line_col(&file_str, location);
            eprintln!("Invalid token at line {}, column {}", line, col);
            std::process::exit(1);
        }
        Err(ParseError::UnrecognizedToken {
            token: (lspan, token, _rspan),
            expected,
        }) => {
            let (line, col) = index_to_line_col(&file_str, lspan);
            eprintln!(
                "Unrecognized token '{}' at line {}, column {}, expected [{}]",
                token,
                line,
                col,
                expected.join(", ")
            );
            std::process::exit(1);
        }
        Err(ParseError::UnrecognizedEOF { location, expected }) => {
            let (line, col) = index_to_line_col(&file_str, location);
            eprintln!(
                "Unexpected EOF at line {}, column {}, expected [{}]",
                line,
                col,
                expected.join(", ")
            );
            std::process::exit(1);
        }
        Err(ParseError::ExtraToken {
            token: (lspan, token, _rspan),
        }) => {
            let (line, col) = index_to_line_col(&file_str, lspan);
            eprintln!(
                "Unexpected extra token '{}' at line {}, column {}",
                token, line, col
            );
            std::process::exit(1);
        }
        Err(ParseError::User { error }) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    }
}

/// Convert an index of the file into a line and column index
fn index_to_line_col(file_str: &str, index: usize) -> (usize, usize) {
    let line = file_str
        .chars()
        .enumerate()
        .take_while(|(i, _)| *i != index)
        .filter(|(_, c)| *c == '\n')
        .count()
        + 1;
    let column = file_str[0..index]
        .chars()
        .rev()
        .take_while(|c| *c != '\n')
        .count()
        + 1;

    (line, column)
}
