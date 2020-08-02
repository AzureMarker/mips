use crate::config::Config;
use crate::constants::{
    FUNCTION_ADD, FUNCTION_ADDU, FUNCTION_BREAK, FUNCTION_SLL, FUNCTION_SYSCALL, OP_ADDI, OP_BEQ,
    OP_JAL, OP_LUI, OP_LW, OP_ORI, OP_R_TYPE, OP_SW, REG_SP, R_DATA_OFFSET, STACK_START,
    TEXT_OFFSET,
};
use crate::instruction::Instruction;
use crate::memory::Memory;
use crate::registers::Registers;

/// A MIPS processor
#[derive(Debug)]
pub struct Processor {
    pub(crate) registers: Registers,
    pub(crate) program_counter: u32,
    pub(crate) next_program_counter: u32,
    pub(crate) memory: Memory,
    pub(crate) config: Config,
    pub running: bool,
}

impl Processor {
    pub fn new(config: Config) -> Self {
        let mut processor = Processor {
            registers: Registers::new(),
            memory: Memory::new(),
            program_counter: 0,
            next_program_counter: 4,
            config,
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

    pub(crate) fn advance_program_counter(&mut self) {
        self.program_counter = self.next_program_counter;
        self.next_program_counter += 4;
    }

    pub fn execute(&mut self, instruction: Instruction) {
        match instruction.op_code() {
            OP_R_TYPE => match instruction.function() {
                FUNCTION_SLL => self.op_sll(instruction),
                FUNCTION_SYSCALL => self.op_syscall(),
                FUNCTION_BREAK => self.op_break(),
                FUNCTION_ADD => self.op_add(instruction),
                FUNCTION_ADDU => self.op_addu(instruction),
                function => panic!("Unknown R-type function 0x{:02x}", function),
            },
            OP_JAL => self.op_jal(instruction),
            OP_BEQ => self.op_beq(instruction),
            OP_ADDI => self.op_addi(instruction),
            OP_ORI => self.op_ori(instruction),
            OP_LUI => self.op_lui(instruction),
            OP_LW => self.op_lw(instruction),
            OP_SW => self.op_sw(instruction),
            op_code => panic!("Unknown op code 0x{:02x}", op_code),
        }
    }
}
