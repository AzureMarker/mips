use crate::constants::REG_RA;
use crate::instruction::Instruction;
use crate::Processor;

impl Processor {
    pub(crate) fn op_jal(&mut self, instruction: Instruction) {
        self.registers.set(REG_RA, self.program_counter + 8);
        let jump_address =
            (0xF0000000 & (self.program_counter + 4)) | (instruction.pseudo_address() << 2);
        println!("jal 0x{:x}", jump_address);

        if self.config.disable_delay_slots {
            self.next_program_counter = jump_address;
            self.advance_program_counter()
        } else {
            self.program_counter = self.next_program_counter;
            self.next_program_counter = jump_address;
        }
    }
}
