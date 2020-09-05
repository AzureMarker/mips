#[macro_use]
extern crate lalrpop_util;

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

    println!("{:#?}", parser::ProgramParser::new().parse(&file_str));

    Ok(())
}
