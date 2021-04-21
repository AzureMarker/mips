use crate::instruction::Instruction;
use crate::Processor;

impl Processor {
    /// Shift left logical
    pub(crate) fn op_sll(&mut self, instruction: Instruction) {
        if instruction.0 == 0 {
            // noop
            self.advance_program_counter();
            return;
        }

        let value = self.registers.get(instruction.t_register()) << instruction.shift_amount();
        self.registers.set(instruction.d_register(), value);
        self.advance_program_counter()
    }

    /// Jump register
    pub(crate) fn op_jr(&mut self, instruction: Instruction) {
        let address = self.registers.get(instruction.s_register());
        self.jump_to(address);
    }

    /// Jump and link register
    pub(crate) fn op_jalr(&mut self, instruction: Instruction) {
        let address = self.registers.get(instruction.s_register());
        let return_register = instruction.d_register();
        let return_address = self.jump_to(address);
        self.registers.set(return_register, return_address);
    }

    /// Break (exceptions/debugger)
    pub(crate) fn op_break(&mut self) {
        self.advance_program_counter();
        self.running = false;
    }

    /// Move from HI
    pub(crate) fn op_mfhi(&mut self, instruction: Instruction) {
        self.registers
            .set(instruction.d_register(), self.registers.hi_register);
        self.advance_program_counter();
    }

    /// Move from LO
    pub(crate) fn op_mflo(&mut self, instruction: Instruction) {
        self.registers
            .set(instruction.d_register(), self.registers.lo_register);
        self.advance_program_counter();
    }

    /// Divide
    pub(crate) fn op_div(&mut self, instruction: Instruction) {
        let s = self.registers.get(instruction.s_register()) as i32;
        let t = self.registers.get(instruction.t_register()) as i32;

        if t == 0 {
            panic!("Divide by zero");
        }

        let quotient = s / t;
        let remainder = s % t;

        self.registers.lo_register = quotient as u32;
        self.registers.hi_register = remainder as u32;
        self.advance_program_counter();
    }

    /// Add (with overflow check)
    pub(crate) fn op_add(&mut self, instruction: Instruction) {
        let a = self.registers.get(instruction.s_register()) as i32;
        let b = self.registers.get(instruction.t_register()) as i32;
        let value = a
            .checked_add(b)
            .unwrap_or_else(|| panic!("Overflow in add"));
        self.registers.set(instruction.d_register(), value as u32);
        self.advance_program_counter();
    }

    /// Add unsigned (no overflow check)
    pub(crate) fn op_addu(&mut self, instruction: Instruction) {
        let a = self.registers.get(instruction.s_register()) as i32;
        let b = self.registers.get(instruction.t_register()) as i32;
        self.registers
            .set(instruction.d_register(), a.wrapping_add(b) as u32);
        self.advance_program_counter();
    }

    /// Subtract (with overflow check)
    pub(crate) fn op_sub(&mut self, instruction: Instruction) {
        let a = self.registers.get(instruction.s_register()) as i32;
        let b = self.registers.get(instruction.t_register()) as i32;
        let value = a
            .checked_sub(b)
            .unwrap_or_else(|| panic!("Overflow in sub"));
        self.registers.set(instruction.d_register(), value as u32);
        self.advance_program_counter();
    }

    /// Bitwise Or
    pub(crate) fn op_or(&mut self, instruction: Instruction) {
        let a = self.registers.get(instruction.s_register());
        let b = self.registers.get(instruction.t_register());
        self.registers.set(instruction.d_register(), a | b);
        self.advance_program_counter();
    }

    /// Set if less than
    pub(crate) fn op_slt(&mut self, instruction: Instruction) {
        let t = self.registers.get(instruction.t_register()) as i32;
        let s = self.registers.get(instruction.s_register()) as i32;
        let result = if s < t { 1 } else { 0 };
        self.registers.set(instruction.d_register(), result);
        self.advance_program_counter();
    }
}
