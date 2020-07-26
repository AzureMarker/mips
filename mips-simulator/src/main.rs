use std::error::Error;
use std::{env, fs};

const OP_R_TYPE: u8 = 0;
const OP_ORI: u8 = 0x0D;

const FUNCTION_ADD: u8 = 0x20;

/// A MIPS processor
#[derive(Debug)]
struct Processor {
    registers: Registers,
}

impl Processor {
    fn new() -> Self {
        Processor {
            registers: Registers::new(),
        }
    }

    fn execute(&mut self, instruction: Instruction) {
        match instruction.op_code() {
            OP_R_TYPE => match instruction.function() {
                FUNCTION_ADD => self.add(instruction),
                function => panic!("Unknown R-type function 0x{:02x}", function),
            },
            OP_ORI => self.ori(instruction),
            op_code => panic!("Unknown op code 0x{:02x}", op_code),
        }
    }

    fn add(&mut self, instruction: Instruction) {
        println!(
            "add {}, {}, {}",
            instruction.d_register(),
            instruction.s_register(),
            instruction.t_register()
        );
        let a = self.registers.get(instruction.s_register());
        let b = self.registers.get(instruction.t_register());
        self.registers.set(instruction.d_register(), a + b);
    }

    fn ori(&mut self, instruction: Instruction) {
        println!(
            "ori {}, {}, {}",
            instruction.t_register(),
            instruction.s_register(),
            instruction.immediate()
        );
        let a = self.registers.get(instruction.s_register());
        let immediate = instruction.immediate() as u32;
        self.registers.set(instruction.t_register(), a | immediate);
    }
}

/// Holds the processor's registers
#[derive(Debug)]
struct Registers([u32; 32]);

impl Registers {
    fn new() -> Self {
        Registers([0; 32])
    }

    fn get(&self, register: u8) -> u32 {
        if register == 0 {
            return 0;
        }

        self.0[register as usize]
    }

    fn set(&mut self, register: u8, value: u32) {
        if register == 0 {
            return;
        }

        self.0[register as usize] = value
    }
}

/// A MIPS instruction
#[derive(Copy, Clone, Debug)]
struct Instruction(u32);

impl Instruction {
    fn instruction_type(&self) -> InstructionType {
        match self.op_code() {
            0 => InstructionType::RType,
            2 | 3 => InstructionType::JType,
            _ => InstructionType::IType,
        }
    }

    /// Get the operation code
    fn op_code(&self) -> u8 {
        ((self.0 & 0xFC000000) >> 26) as u8
    }

    /// Get the s register (for R and I type instructions)
    fn s_register(&self) -> u8 {
        ((self.0 & 0x03E00000) >> 21) as u8
    }

    /// Get the t register (for R and I type instructions)
    fn t_register(&self) -> u8 {
        ((self.0 & 0x001F0000) >> 16) as u8
    }

    /// Get the d register (For R type instructions)
    fn d_register(&self) -> u8 {
        ((self.0 & 0x0000F800) >> 11) as u8
    }

    /// Get the shift amount (for R type instructions)
    fn shift_amount(&self) -> u8 {
        ((self.0 & 0x000007C0) >> 6) as u8
    }

    /// Get the ALU function (for R type instructions)
    fn function(&self) -> u8 {
        (self.0 & 0x0000003F) as u8
    }

    /// Get the immediate value (for I type instructions)
    fn immediate(&self) -> u16 {
        (self.0 & 0x0000FFFF) as u16
    }

    /// Get the pseudo address (for J type instructions)
    fn pseudo_address(&self) -> u32 {
        self.0 & 0x03FFFFFF
    }
}

#[derive(Debug)]
enum InstructionType {
    RType,
    IType,
    JType,
}

/// RSIM's module header
struct ModuleHeader {
    magic: u16,
    version: u16,
    flags: u32,
    entry: u32,
    section_sizes: [u32; 10],
}

fn main() -> Result<(), Box<dyn Error>> {
    let file_path = env::args().nth(1).expect("Must provide a file argument");
    let file_data = fs::read(file_path)?;

    let mut processor = Processor::new();
    let instructions: Vec<_> = file_data
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
