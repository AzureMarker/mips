use crate::constants::REG_RA;
use crate::instruction::Instruction;
use crate::Processor;

impl Processor {
    pub(crate) fn op_j(&mut self, instruction: Instruction) {
        let address = self.parse_pseudo_address(instruction.pseudo_address());
        debug!("j 0x{:x}", address);

        self.jump_to(address);
    }

    pub(crate) fn op_jal(&mut self, instruction: Instruction) {
        self.registers.set(REG_RA, self.program_counter + 8);
        let address = self.parse_pseudo_address(instruction.pseudo_address());
        debug!("jal 0x{:x}", address);

        self.jump_to(address);
    }

    fn parse_pseudo_address(&self, pseudo_address: u32) -> u32 {
        (0xF0000000 & (self.program_counter + 4)) | (pseudo_address << 2)
    }
}
