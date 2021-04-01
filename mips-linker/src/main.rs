use crate::load_module::obj_to_load_module;
use env_logger::Env;
use mips_types::module::R2KModule;
use std::error::Error;
use std::fs::File;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::{fs, io};
use structopt::StructOpt;

mod load_module;
mod references;
mod relocation;
mod util;

#[derive(StructOpt)]
struct CliArgs {
    #[structopt(parse(from_os_str), required = true)]
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

    link_objects(&args.object_files, &args.output_file)?;

    Ok(())
}

fn link_objects(obj_file_paths: &[PathBuf], output_file_path: &Path) -> io::Result<()> {
    // FIXME: allow more than one object file
    assert_eq!(obj_file_paths.len(), 1);

    // Load the object files
    let mut obj_files = obj_file_paths
        .iter()
        .map(|obj_file_path| {
            let file_data = fs::read(obj_file_path)?;
            R2KModule::parse(&mut Cursor::new(file_data))
        })
        .collect::<io::Result<Vec<_>>>()?;

    log::trace!(
        "Loaded object files: {:#?}",
        obj_files.iter().map(|obj| &obj.header).collect::<Vec<_>>()
    );
    log::info!("Loaded {} object files", obj_files.len());

    let load_module = obj_to_load_module(obj_files.remove(0));

    // Write out load module
    let mut output_file = File::create(output_file_path)?;
    load_module.write(&mut output_file)?;
    log::info!("Wrote load module to {}", output_file_path.display());

    Ok(())
}
