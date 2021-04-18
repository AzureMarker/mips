use crate::config::Config;
use crate::instruction::Instruction;
use crate::memory::Memory;
use crate::registers::Registers;
use mips_types::constants::*;
use mips_types::module::R2KModule;

/// A MIPS processor
#[derive(Debug)]
pub struct Processor {
    pub registers: Registers,
    pub program_counter: u32,
    pub(crate) next_program_counter: u32,
    pub(crate) memory: Memory,
    pub(crate) config: Config,
    pub running: bool,
    pub return_code: i32,
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
            return_code: 0,
        };
        processor.registers.set(REG_SP, STACK_BOTTOM);
        processor
    }

    /// Load an RSIM executable module into memory and prepare for execution
    pub fn load_rsim_module(&mut self, module: &R2KModule) {
        self.memory
            .load_into_memory(&module.text_section, TEXT_OFFSET);

        let mut data_offset = DATA_OFFSET;
        self.memory
            .load_into_memory(&module.rdata_section, DATA_OFFSET);
        data_offset += module.rdata_section.len() as u32;
        self.memory
            .load_into_memory(&module.data_section, data_offset);
        data_offset += module.data_section.len() as u32;
        self.memory
            .load_into_memory(&module.sdata_section, data_offset);

        self.set_program_counter(module.header.entry);
    }

    fn set_program_counter(&mut self, address: u32) {
        self.program_counter = address;
        self.next_program_counter = address + 4;
    }

    /// Execute the next instruction
    pub fn step(&mut self) {
        let instruction = self.load_next_instruction();
        trace!("{:08x?}", instruction);
        debug!("{}", instruction.stringify(self.program_counter));
        self.execute(instruction);
        trace!("{:#08x?}", self);
    }

    pub fn load_next_instruction(&self) -> Instruction {
        trace!("Loading instruction at 0x{:x}", self.program_counter);
        Instruction(self.memory.get_word(self.program_counter))
    }

    /// Update the program counter with the queued address, and advance the
    /// queued address by 4 bytes.
    pub(crate) fn advance_program_counter(&mut self) {
        self.program_counter = self.next_program_counter;
        self.next_program_counter += 4;
    }

    /// Jump to an address. If delay slots are enabled, the next instruction
    /// will be executed before the jump. Otherwise, the jump is immediate.
    /// The return address (next instruction that would have happened if not
    /// for the jump) is returned.
    pub(crate) fn jump_to(&mut self, address: u32) -> u32 {
        if self.config.enable_delay_slots {
            self.program_counter = self.next_program_counter;
            self.next_program_counter = address;
            self.program_counter + 4
        } else {
            let return_address = self.program_counter + 4;
            self.next_program_counter = address;
            self.advance_program_counter();
            return_address
        }
    }

    /// Execute an instruction
    pub fn execute(&mut self, instruction: Instruction) {
        match instruction.op_code() {
            OP_R_TYPE => match instruction.function() {
                FUNCTION_SLL => self.op_sll(instruction),
                FUNCTION_JR => self.op_jr(instruction),
                FUNCTION_JALR => self.op_jalr(instruction),
                FUNCTION_SYSCALL => self.op_syscall(),
                FUNCTION_BREAK => self.op_break(),
                FUNCTION_MFHI => self.op_mfhi(instruction),
                FUNCTION_MFLO => self.op_mflo(instruction),
                FUNCTION_DIV => self.op_div(instruction),
                FUNCTION_ADD => self.op_add(instruction),
                FUNCTION_ADDU => self.op_addu(instruction),
                FUNCTION_SUB => self.op_sub(instruction),
                FUNCTION_OR => self.op_or(instruction),
                FUNCTION_SLT => self.op_slt(instruction),
                function => panic!("Unknown R-type function 0x{:02x}", function),
            },
            OP_J => self.op_j(instruction),
            OP_JAL => self.op_jal(instruction),
            OP_BEQ => self.op_beq(instruction),
            OP_BNE => self.op_bne(instruction),
            OP_ADDI => self.op_addi(instruction),
            OP_ADDIU => self.op_addiu(instruction),
            OP_SLTI => self.op_slti(instruction),
            OP_ORI => self.op_ori(instruction),
            OP_LUI => self.op_lui(instruction),
            OP_LB => self.op_lb(instruction),
            OP_LW => self.op_lw(instruction),
            OP_SB => self.op_sb(instruction),
            OP_SW => self.op_sw(instruction),
            op_code => panic!("Unknown op code 0x{:02x}", op_code),
        }
    }
}
