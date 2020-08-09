use crate::constants::REG_RA;
use crate::instruction::Instruction;
use crate::Processor;

impl Processor {
    /// Jump
    pub(crate) fn op_j(&mut self, instruction: Instruction) {
        self.jump_to(instruction.real_address(self.program_counter));
    }

    /// Jump and link
    pub(crate) fn op_jal(&mut self, instruction: Instruction) {
        let offset = if self.config.disable_delay_slots {
            4
        } else {
            8
        };

        self.registers.set(REG_RA, self.program_counter + offset);
        self.jump_to(instruction.real_address(self.program_counter));
    }
}
