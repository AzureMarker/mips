use mips_simulator::config::Config;
use mips_simulator::rsim::RsimModule;
use mips_simulator::Processor;
use std::error::Error;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
struct CliArgs {
    #[structopt(long)]
    disable_delay_slots: bool,

    #[structopt(parse(from_os_str))]
    file_path: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = CliArgs::from_args();
    let file_data = fs::read(&args.file_path)?;
    let module = RsimModule::parse(&mut Cursor::new(file_data))?;
    println!("Loaded module with header: {:?}", module.header);

    let mut processor = Processor::new(Config {
        disable_delay_slots: args.disable_delay_slots,
    });
    processor.text_segment(module.text_section());
    processor.read_only_data_segment(module.read_only_data_section());
    processor.set_entry(module.header.entry);

    while processor.running {
        processor.step();
    }

    Ok(())
}
