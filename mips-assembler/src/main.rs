#[macro_use]
extern crate lalrpop_util;

use lalrpop_util::ParseError;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

lalrpop_mod!(parser);

mod ast;

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

    let exit_code;
    match parser::ProgramParser::new().parse(&file_str) {
        Ok(parsed_ast) => {
            println!("{:#?}", parsed_ast);
            exit_code = 0;
        }
        Err(ParseError::InvalidToken { location }) => {
            let (line, col) = index_to_line_col(&file_str, location);
            eprintln!("Invalid token at line {}, column {}", line, col);
            exit_code = 1;
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
            exit_code = 1;
        }
        Err(ParseError::UnrecognizedEOF { location, expected }) => {
            let (line, col) = index_to_line_col(&file_str, location);
            eprintln!(
                "Unexpected EOF at line {}, column {}, expected [{}]",
                line,
                col,
                expected.join(", ")
            );
            exit_code = 1;
        }
        Err(ParseError::ExtraToken {
            token: (lspan, token, _rspan),
        }) => {
            let (line, col) = index_to_line_col(&file_str, lspan);
            eprintln!(
                "Unexpected extra token '{}' at line {}, column {}",
                token, line, col
            );
            exit_code = 1;
        }
        Err(ParseError::User { error }) => {
            eprintln!("{}", error);
            exit_code = 1;
        }
    }

    std::process::exit(exit_code);
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
