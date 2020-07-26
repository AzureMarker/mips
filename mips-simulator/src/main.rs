use mips_simulator::rsim::RsimModule;
use mips_simulator::{Instruction, Processor};
use std::error::Error;
use std::io::Cursor;
use std::{env, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let file_path = env::args().nth(1).expect("Must provide a file argument");
    let file_data = fs::read(file_path)?;
    let module = RsimModule::parse(&mut Cursor::new(file_data))?;

    let mut processor = Processor::new();
    let instructions: Vec<_> = module
        .text_section()
        .chunks_exact(4)
        .map(|chunk| {
            let chunk: [u8; 4] = [chunk[0], chunk[1], chunk[2], chunk[3]];
            Instruction(u32::from_be_bytes(chunk))
        })
        .collect();

    for instruction in instructions {
        println!("{:08x?}", instruction);
        processor.execute(instruction);
        println!("{:#08x?}", processor);
    }

    Ok(())
}
