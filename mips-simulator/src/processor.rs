use crate::constants::{
    FUNCTION_ADD, FUNCTION_BREAK, FUNCTION_SLL, OP_ADDI, OP_JAL, OP_LW, OP_ORI, OP_R_TYPE, OP_SW,
    REG_RA, REG_SP, R_DATA_OFFSET, STACK_START, TEXT_OFFSET,
};
use crate::instruction::Instruction;
use crate::math::add_unsigned;
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

    pub fn read_only_data_segment(&mut self, data: &[u8]) {
        self.memory.load_into_memory(data, R_DATA_OFFSET);
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
                FUNCTION_SLL => self.op_sll(instruction),
                FUNCTION_ADD => self.op_add(instruction),
                FUNCTION_BREAK => self.op_break(),
                function => panic!("Unknown R-type function 0x{:02x}", function),
            },
            OP_JAL => self.op_jal(instruction),
            OP_ADDI => self.op_addi(instruction),
            OP_ORI => self.op_ori(instruction),
            OP_LW => self.op_lw(instruction),
            OP_SW => self.op_sw(instruction),
            op_code => panic!("Unknown op code 0x{:02x}", op_code),
        }
    }

    fn op_sll(&mut self, instruction: Instruction) {
        if instruction.0 == 0 {
            println!("noop");
            self.advance_program_counter();
            return;
        }

        println!(
            "sll ${}, ${}, {}",
            instruction.d_register(),
            instruction.t_register(),
            instruction.shift_amount()
        );
        let value = self.registers.get(instruction.t_register()) << instruction.shift_amount();
        self.registers.set(instruction.d_register(), value);
        self.advance_program_counter()
    }

    fn op_add(&mut self, instruction: Instruction) {
        println!(
            "add ${}, ${}, ${}",
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

    fn op_addi(&mut self, instruction: Instruction) {
        println!(
            "addi ${}, ${}, {}",
            instruction.t_register(),
            instruction.s_register(),
            instruction.immediate()
        );
        let value = add_unsigned(
            self.registers.get(instruction.s_register()),
            instruction.immediate() as i32,
        );
        self.registers.set(instruction.t_register(), value);
        self.advance_program_counter()
    }

    fn op_ori(&mut self, instruction: Instruction) {
        println!(
            "ori ${}, ${}, 0x{:x}",
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
            .get_word(add_unsigned(s_address, instruction.immediate() as i32));
        self.registers.set(instruction.t_register(), value);
        self.advance_program_counter();
    }

    fn op_sw(&mut self, instruction: Instruction) {
        println!(
            "sw ${}, {}(${})",
            instruction.t_register(),
            instruction.immediate(),
            instruction.s_register()
        );
        let s_address = self.registers.get(instruction.s_register());
        let address = add_unsigned(s_address, instruction.immediate() as i32);
        let value = self.registers.get(instruction.t_register());
        self.memory.set_word(address, value);
        self.advance_program_counter();
    }
}
