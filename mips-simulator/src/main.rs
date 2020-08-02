use mips_simulator::config::Config;
use mips_simulator::rsim::RsimModule;
use mips_simulator::Processor;
use std::error::Error;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use structopt::StructOpt;

#[macro_use]
extern crate log;

#[derive(StructOpt)]
struct CliArgs {
    /// Disables jump/branch delay slots. RSIM code requires this option.
    #[structopt(long)]
    disable_delay_slots: bool,

    #[structopt(parse(from_os_str))]
    file_path: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Setup logging and parse CLI args
    env_logger::init();
    let args = CliArgs::from_args();

    // Load the executable module
    let file_data = fs::read(&args.file_path)?;
    let module = RsimModule::parse(&mut Cursor::new(file_data))?;
    info!("Loaded module with header: {:?}", module.header);

    // Setup the processor
    let mut processor = Processor::new(Config {
        disable_delay_slots: args.disable_delay_slots,
    });
    processor.load_rsim_module(&module);

    // Run the code
    while processor.running {
        processor.step();
    }

    std::process::exit(processor.return_code);
}
