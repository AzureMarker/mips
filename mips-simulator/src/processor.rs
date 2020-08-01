use crate::constants::{
    FUNCTION_ADD, FUNCTION_BREAK, OP_JAL, OP_LW, OP_ORI, OP_R_TYPE, REG_RA, REG_SP, STACK_START,
    TEXT_OFFSET,
};
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
        let mut processor = Processor {
            registers: Registers::new(),
            memory: Memory::new(),
            program_counter: 0,
            next_program_counter: 4,
            running: true,
        };
        processor.registers.set(REG_SP, STACK_START);
        processor
    }

    pub fn text_segment(&mut self, data: &[u8]) {
        self.memory.load_into_memory(data, TEXT_OFFSET);
    }

    pub fn set_entry(&mut self, address: u32) {
        self.program_counter = address;
        self.next_program_counter = address + 4;
    }

    pub fn step(&mut self) {
        let instruction = self.load_next_instruction();
        println!("{:08x?}", instruction);
        self.execute(instruction);
        println!("{:#08x?}", self);
    }

    fn load_next_instruction(&self) -> Instruction {
        println!("Loading instruction at 0x{:x}", self.program_counter);
        Instruction(self.memory.get_word(self.program_counter))
    }

    fn advance_program_counter(&mut self) {
        self.program_counter = self.next_program_counter;
        self.next_program_counter += 4;
    }

    pub fn execute(&mut self, instruction: Instruction) {
        match instruction.op_code() {
            OP_R_TYPE => match instruction.function() {
                FUNCTION_ADD => self.op_add(instruction),
                FUNCTION_BREAK => self.op_break(),
                function => panic!("Unknown R-type function 0x{:02x}", function),
            },
            OP_JAL => self.op_jal(instruction),
            OP_ORI => self.op_ori(instruction),
            OP_LW => self.op_lw(instruction),
            op_code => panic!("Unknown op code 0x{:02x}", op_code),
        }
    }

    fn op_add(&mut self, instruction: Instruction) {
        println!(
            "add {}, {}, {}",
            instruction.d_register(),
            instruction.s_register(),
            instruction.t_register()
        );
        let a = self.registers.get(instruction.s_register());
        let b = self.registers.get(instruction.t_register());
        self.registers.set(instruction.d_register(), a + b);
        self.advance_program_counter();
    }

    fn op_break(&mut self) {
        println!("Executing a break instruction");
        self.advance_program_counter();
        self.running = false;
    }

    fn op_jal(&mut self, instruction: Instruction) {
        self.registers.set(REG_RA, self.program_counter + 8);
        let jump_address =
            (0xF0000000 & (self.program_counter + 4)) | (instruction.pseudo_address() << 2);
        println!("jal 0x{:x}", jump_address);
        self.program_counter = self.next_program_counter;
        self.next_program_counter = jump_address;
    }

    fn op_ori(&mut self, instruction: Instruction) {
        println!(
            "ori {}, {}, {}",
            instruction.t_register(),
            instruction.s_register(),
            instruction.immediate()
        );
        let a = self.registers.get(instruction.s_register());
        let immediate = instruction.immediate() as u32;
        self.registers.set(instruction.t_register(), a | immediate);
        self.advance_program_counter();
    }

    fn op_lw(&mut self, instruction: Instruction) {
        println!(
            "lw ${}, {}(${})",
            instruction.t_register(),
            instruction.immediate(),
            instruction.s_register()
        );
        let s_address = self.registers.get(instruction.s_register());
        let value = self
            .memory
            .get_word(s_address + instruction.immediate() as u32);
        self.registers.set(instruction.t_register(), value);
        self.advance_program_counter();
    }
}
