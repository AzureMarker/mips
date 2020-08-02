use crate::instruction::Instruction;
use crate::math::add_unsigned;
use crate::Processor;

impl Processor {
    pub(crate) fn op_beq(&mut self, instruction: Instruction) {
        let offset = (instruction.immediate() as i32) << 2;
        let jump_address = add_unsigned(self.next_program_counter, offset);
        debug!(
            "beq ${}, ${}, 0x{:x}",
            instruction.s_register(),
            instruction.t_register(),
            jump_address
        );

        let s_value = self.registers.get(instruction.s_register());
        let t_value = self.registers.get(instruction.t_register());

        if s_value == t_value {
            if self.config.disable_delay_slots {
                self.next_program_counter = jump_address;
                self.advance_program_counter();
            } else {
                self.program_counter = self.next_program_counter;
                self.next_program_counter = jump_address;
            }
        } else {
            self.advance_program_counter();
        }
    }

    pub(crate) fn op_addi(&mut self, instruction: Instruction) {
        debug!(
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

    pub(crate) fn op_ori(&mut self, instruction: Instruction) {
        debug!(
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

    pub(crate) fn op_lui(&mut self, instruction: Instruction) {
        debug!(
            "lui ${}, 0x{:x}",
            instruction.t_register(),
            instruction.immediate()
        );
        let value = (instruction.immediate() as u32) << 16;
        self.registers.set(instruction.t_register(), value);
        self.advance_program_counter();
    }

    pub(crate) fn op_lw(&mut self, instruction: Instruction) {
        debug!(
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

    pub(crate) fn op_sw(&mut self, instruction: Instruction) {
        debug!(
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
