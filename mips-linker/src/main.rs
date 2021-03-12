use env_logger::Env;
use mips_types::module::R2KModule;
use std::error::Error;
use std::io::{Cursor, Write};
use std::path::PathBuf;
use std::{fs, io};
use structopt::StructOpt;

#[derive(StructOpt)]
struct CliArgs {
    #[structopt(parse(from_os_str))]
    object_files: Vec<PathBuf>,

    #[structopt(
        parse(from_os_str),
        long = "output",
        short = "o",
        default_value = "out.out"
    )]
    output_file: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Setup logging and parse CLI args
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .init();
    let args = CliArgs::from_args();

    link_objects(&args.object_files)?;

    Ok(())
}

fn link_objects(obj_file_paths: &[PathBuf]) -> io::Result<()> {
    // Load the object files
    let obj_files = obj_file_paths
        .iter()
        .map(|obj_file_path| {
            let file_data = fs::read(obj_file_path)?;
            R2KModule::parse(&mut Cursor::new(file_data))
        })
        .collect::<io::Result<Vec<_>>>()?;

    log::trace!("Loaded object files: {:#?}", obj_files);
    log::info!("Loaded {} object files", obj_files.len());

    Ok(())
}
