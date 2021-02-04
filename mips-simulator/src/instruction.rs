use mips_types::constants::{
    FUNCTION_ADD, FUNCTION_ADDU, FUNCTION_BREAK, FUNCTION_JR, FUNCTION_SLL, FUNCTION_SYSCALL,
    OP_ADDI, OP_BEQ, OP_J, OP_JAL, OP_LUI, OP_LW, OP_ORI, OP_R_TYPE, OP_SLTI, OP_SW,
    REGISTER_NAMES,
};
use std::fmt;
use std::fmt::{Display, Formatter};

/// A MIPS instruction
#[derive(Copy, Clone, Debug)]
pub struct Instruction(pub u32);

impl Instruction {
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

    /// Convert the pseudo address to a full address.
    /// The first four bits are taken from the program counter, and the lower
    /// two bits are zeros.
    pub fn real_address(&self, program_counter: u32) -> u32 {
        (0xF0000000 & (program_counter + 4)) | (self.pseudo_address() << 2)
    }

    /// Decode and format the instruction
    pub fn stringify(&self, program_counter: u32) -> String {
        match self.op_code() {
            OP_R_TYPE => match self.function() {
                FUNCTION_SLL => {
                    if self.0 == 0 {
                        "noop".to_string()
                    } else {
                        format!(
                            "sll {}, {}, {}",
                            Register(self.d_register()),
                            Register(self.t_register()),
                            self.shift_amount()
                        )
                    }
                }
                FUNCTION_JR => format!("jr {}", Register(self.s_register())),
                FUNCTION_SYSCALL => "syscall".to_string(),
                FUNCTION_BREAK => "break".to_string(),
                FUNCTION_ADD => format!(
                    "add {}, {}, {}",
                    Register(self.d_register()),
                    Register(self.s_register()),
                    Register(self.t_register())
                ),
                FUNCTION_ADDU => format!(
                    "addu {}, {}, {}",
                    Register(self.d_register()),
                    Register(self.s_register()),
                    Register(self.t_register())
                ),
                function => panic!("Unknown R-type function 0x{:02x}", function),
            },
            OP_J => format!("j 0x{:x}", self.real_address(program_counter)),
            OP_JAL => format!("jal 0x{:x}", self.real_address(program_counter)),
            OP_BEQ => format!(
                "beq {}, {}, {}",
                Register(self.s_register()),
                Register(self.t_register()),
                self.immediate()
            ),
            OP_ADDI => format!(
                "addi {}, {}, {}",
                Register(self.t_register()),
                Register(self.s_register()),
                self.immediate()
            ),
            OP_SLTI => format!(
                "slti {}, {}, {}",
                Register(self.t_register()),
                Register(self.s_register()),
                self.immediate()
            ),
            OP_ORI => format!(
                "ori {}, {}, 0x{:x}",
                Register(self.t_register()),
                Register(self.s_register()),
                self.immediate() as u16
            ),
            OP_LUI => format!(
                "lui {}, 0x{:x}",
                Register(self.t_register()),
                self.immediate() as u16
            ),
            OP_LW => format!(
                "lw {}, {}({})",
                Register(self.t_register()),
                self.immediate(),
                Register(self.s_register())
            ),
            OP_SW => format!(
                "sw {}, {}({})",
                Register(self.t_register()),
                self.immediate(),
                Register(self.s_register())
            ),
            op_code => panic!("Unknown op code 0x{:02x}", op_code),
        }
    }
}

/// Pretty-print the register using its name
struct Register(u8);

impl Display for Register {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        assert!(self.0 < 32);

        f.write_str(REGISTER_NAMES[self.0 as usize])
    }
}
