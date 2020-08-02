use crate::constants::REG_RA;
use crate::instruction::Instruction;
use crate::Processor;

impl Processor {
    /// Jump
    pub(crate) fn op_j(&mut self, instruction: Instruction) {
        let address = self.parse_pseudo_address(instruction.pseudo_address());
        debug!("j 0x{:x}", address);

        self.jump_to(address);
    }

    /// Jump and link
    pub(crate) fn op_jal(&mut self, instruction: Instruction) {
        let offset = if self.config.disable_delay_slots {
            4
        } else {
            8
        };
        self.registers.set(REG_RA, self.program_counter + offset);
        let address = self.parse_pseudo_address(instruction.pseudo_address());
        debug!("jal 0x{:x}", address);

        self.jump_to(address);
    }

    /// Convert a pseudo address to a full address.
    /// The first four bits are taken from the program counter, and the lower
    /// two bits are zeros.
    fn parse_pseudo_address(&self, pseudo_address: u32) -> u32 {
        (0xF0000000 & (self.program_counter + 4)) | (pseudo_address << 2)
    }
}
