/// A MIPS instruction
#[derive(Copy, Clone, Debug)]
pub struct Instruction(pub u32);

#[derive(Debug)]
pub enum InstructionType {
    RType,
    IType,
    JType,
}

impl Instruction {
    pub fn instruction_type(&self) -> InstructionType {
        match self.op_code() {
            0 => InstructionType::RType,
            2 | 3 => InstructionType::JType,
            _ => InstructionType::IType,
        }
    }

    /// Get the operation code
    pub fn op_code(&self) -> u8 {
        ((self.0 & 0xFC000000) >> 26) as u8
    }

    /// Get the s register (for R and I type instructions)
    pub fn s_register(&self) -> u8 {
        ((self.0 & 0x03E00000) >> 21) as u8
    }

    /// Get the t register (for R and I type instructions)
    pub fn t_register(&self) -> u8 {
        ((self.0 & 0x001F0000) >> 16) as u8
    }

    /// Get the d register (For R type instructions)
    pub fn d_register(&self) -> u8 {
        ((self.0 & 0x0000F800) >> 11) as u8
    }

    /// Get the shift amount (for R type instructions)
    pub fn shift_amount(&self) -> u8 {
        ((self.0 & 0x000007C0) >> 6) as u8
    }

    /// Get the ALU function (for R type instructions)
    pub fn function(&self) -> u8 {
        (self.0 & 0x0000003F) as u8
    }

    /// Get the immediate value (for I type instructions)
    pub fn immediate(&self) -> i16 {
        (self.0 & 0x0000FFFF) as i16
    }

    /// Get the pseudo address (for J type instructions)
    pub fn pseudo_address(&self) -> u32 {
        self.0 & 0x03FFFFFF
    }
}
