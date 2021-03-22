use crate::references::resolve_references;
use crate::relocation::relocate;
use crate::util::{make_symbol_table, R2KStrings};
use env_logger::Env;
use mips_types::constants::{DATA_OFFSET, TEXT_OFFSET};
use mips_types::module::{R2KModule, R2KModuleHeader, R2KVersion, R2K_MAGIC, RELOCATION_INDEX};
use std::error::Error;
use std::fs::File;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::{fs, io};
use structopt::StructOpt;

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

fn obj_to_load_module(obj_module: R2KModule) -> R2KModule {
    let entry = TEXT_OFFSET;
    let mut section_sizes = obj_module.header.section_sizes;
    let mut relocation = obj_module.relocation_section;
    let mut references = obj_module.reference_section;
    let mut text_section = obj_module.text_section;
    let mut data_section = obj_module.data_section;
    let strings = R2KStrings::new(&obj_module.string_table);
    let symbols = make_symbol_table(strings, &obj_module.symbol_table);

    relocate(&mut text_section, 1, TEXT_OFFSET, &mut relocation);
    relocate(&mut data_section, 3, DATA_OFFSET, &mut relocation);
    resolve_references(&mut text_section, 1, strings, &symbols, &mut references);
    resolve_references(&mut data_section, 3, strings, &symbols, &mut references);

    // FIXME: refactor and support all relocatable sections
    assert!(
        relocation.is_empty(),
        "Only text and data relocation is currently supported"
    );

    section_sizes[RELOCATION_INDEX] = relocation.len() as u32;

    R2KModule {
        header: R2KModuleHeader {
            magic: R2K_MAGIC,
            version: R2KVersion::Version1,
            flags: 0,
            entry,
            section_sizes,
        },
        text_section,
        rdata_section: obj_module.rdata_section,
        data_section,
        sdata_section: obj_module.sdata_section,
        sbss_size: obj_module.sbss_size,
        bss_size: obj_module.bss_size,
        relocation_section: relocation,
        reference_section: references,
        symbol_table: obj_module.symbol_table,
        string_table: obj_module.string_table,
    }
}
