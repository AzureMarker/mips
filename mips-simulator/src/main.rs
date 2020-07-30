use mips_simulator::rsim::RsimModule;
use mips_simulator::{Instruction, Processor};
use std::error::Error;
use std::io::Cursor;
use std::{env, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let file_path = env::args().nth(1).expect("Must provide a file argument");
    let file_data = fs::read(file_path)?;
    let module = RsimModule::parse(&mut Cursor::new(file_data))?;
    println!("Loaded module with header: {:?}", module.header);

    let mut processor = Processor::new();
    processor.load_into_memory(module.text_section(), module.header.entry);
    processor.program_counter = module.header.entry;

    while processor.running {
        processor.step();
    }

    Ok(())
}
