#[macro_use]
extern crate lalrpop_util;

use crate::ast::{Program, Span};
use env_logger::Env;
use lalrpop_util::ParseError;
use std::error::Error;
use std::fs::File;
use std::io::Write;
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
mod lower_ir_instruction;
mod string_table;
mod string_unescape;

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
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .init();
    let args = CliArgs::from_args();

    // Load the assembly file
    let file_str = fs::read_to_string(&args.input_file)?;
    let program = parse(&file_str);

    assemble_file(program, &file_str, args)?;

    Ok(())
}

fn assemble_file(program: Program, file_str: &str, args: CliArgs) -> io::Result<()> {
    log::trace!("{:#?}", program);
    let program_ir = match program.lower() {
        Ok(ir) => ir,
        Err(e) => exit_with_error(file_str, e.span(), "message_goes_here"),
    };
    log::trace!("{:#?}", program_ir);
    let program_mips = program_ir.lower();
    log::trace!("{:#x?}", program_mips);

    let mut output = File::create(&args.output_file)?;
    program_mips.write(&mut output)
}

fn exit_with_error(file_str: &str, span: Span, message: &str) -> ! {
    let (line_start, col_start) = index_to_line_col(file_str, span.0);
    let (_, col_end) = index_to_line_col(file_str, span.1);
    let line = file_str
        .lines()
        .nth(line_start - 1)
        .expect("Out of bounds line");
    let underline: String = std::iter::repeat(' ')
        .take(col_start - 1)
        .chain(std::iter::repeat('~').take(col_end - col_start))
        .collect();
    log::error!(
        "Error at line {} column {}:\n| {}\n| {}\nerror: {}",
        line_start,
        col_start,
        line,
        underline,
        message
    );
    std::process::exit(1);
}

/// Parse the MIPS program. If there are errors during parsing, the program will
/// exit.
fn parse(file_str: &str) -> Program {
    match parser::ProgramParser::new().parse(file_str) {
        Ok(parsed_ast) => parsed_ast,
        Err(ParseError::InvalidToken { location }) => {
            let (line, col) = index_to_line_col(&file_str, location);
            log::error!("Invalid token at line {}, column {}", line, col);
            std::process::exit(1);
        }
        Err(ParseError::UnrecognizedToken {
            token: (lspan, token, _rspan),
            expected,
        }) => {
            let (line, col) = index_to_line_col(&file_str, lspan);
            log::error!(
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
            log::error!(
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
            log::error!(
                "Unexpected extra token '{}' at line {}, column {}",
                token,
                line,
                col
            );
            std::process::exit(1);
        }
        Err(ParseError::User { error }) => {
            log::error!("{}", error);
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
