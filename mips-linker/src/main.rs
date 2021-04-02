use crate::load_module::obj_to_load_module;
use env_logger::Env;
use mips_types::module::R2KModule;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{Cursor, Write};
use std::os::unix::fs::OpenOptionsExt;
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

    let output_module = obj_to_load_module(obj_files.remove(0));
    let is_load_module = output_module.is_load_module();

    // Write out the module
    let mut output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        // Make load modules executable
        .mode(if is_load_module { 0o755 } else { 0o644 })
        .open(output_file_path)?;
    output_module.write(&mut output_file)?;

    log::info!(
        "Wrote {} module to {}",
        if is_load_module { "load" } else { "object" },
        output_file_path.display()
    );

    Ok(())
}
