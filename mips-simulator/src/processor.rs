use crate::constants::{FUNCTION_ADD, FUNCTION_BREAK, OP_ORI, OP_R_TYPE};
use crate::instruction::Instruction;
use crate::memory::Memory;
use crate::registers::Registers;

/// A MIPS processor
#[derive(Debug)]
pub struct Processor {
    registers: Registers,
    program_counter: u32,
    next_program_counter: u32,
    memory: Memory,
    pub running: bool,
}

impl Processor {
    pub fn new() -> Self {
        Processor {
            registers: Registers::new(),
            memory: Memory::new(),
            program_counter: 0,
            next_program_counter: 4,
            running: true,
        }
    }

    pub fn load_into_memory(&mut self, data: &[u8], offset: u32) {
        self.memory.load_into_memory(data, offset);
    }

    pub fn set_entry(&mut self, address: u32) {
        self.program_counter = address;
        self.next_program_counter = address + 4;
    }

    pub fn step(&mut self) {
        let instruction = self.load_next_instruction();
        println!("{:08x?}", instruction);
        self.execute(instruction);
        self.advance_program_counter();
        println!("{:#08x?}", self);
    }

    fn load_next_instruction(&self) -> Instruction {
        let pc = self.program_counter;
        let bytes = &self.memory.get_range(pc..(pc + 4));
        let bytes: [u8; 4] = [bytes[0], bytes[1], bytes[2], bytes[3]];

        Instruction(u32::from_be_bytes(bytes))
    }

    fn advance_program_counter(&mut self) {
        self.program_counter = self.next_program_counter;
        self.next_program_counter += 4;
    }

    pub fn execute(&mut self, instruction: Instruction) {
        match instruction.op_code() {
            OP_R_TYPE => match instruction.function() {
                FUNCTION_ADD => self.add(instruction),
                FUNCTION_BREAK => self.break_fn(),
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

    fn break_fn(&mut self) {
        println!("Executing a break instruction");
        self.running = false;
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
