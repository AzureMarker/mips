use crate::instruction::Instruction;
use crate::Processor;

impl Processor {
    pub(crate) fn op_sll(&mut self, instruction: Instruction) {
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

    pub(crate) fn op_break(&mut self) {
        println!("Executing a break instruction");
        self.advance_program_counter();
        self.running = false;
    }

    pub(crate) fn op_add(&mut self, instruction: Instruction) {
        println!(
            "add ${}, ${}, ${}",
            instruction.d_register(),
            instruction.s_register(),
            instruction.t_register()
        );
        let a = self.registers.get(instruction.s_register());
        let b = self.registers.get(instruction.t_register());
        self.registers
            .set(instruction.d_register(), a.wrapping_add(b));
        self.advance_program_counter();
    }

    pub(crate) fn op_addu(&mut self, instruction: Instruction) {
        println!(
            "addu ${}, ${}, ${}",
            instruction.d_register(),
            instruction.s_register(),
            instruction.t_register()
        );
        let a = self.registers.get(instruction.s_register());
        let b = self.registers.get(instruction.t_register());
        let value = a
            .checked_add(b)
            .unwrap_or_else(|| panic!("Overflow in addu"));
        self.registers.set(instruction.d_register(), value);
        self.advance_program_counter();
    }
}
