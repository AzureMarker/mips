use mips_types::constants::*;
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
        // Shorthand functions to stringify common instruction forms.
        let dst = |name: &str| {
            format!(
                "{} {}, {}, {}",
                name,
                Register(self.d_register()),
                Register(self.s_register()),
                Register(self.t_register())
            )
        };
        let sti = |name: &str| {
            format!(
                "{} {}, {}, {}",
                name,
                Register(self.s_register()),
                Register(self.t_register()),
                self.immediate()
            )
        };
        let tis = |name: &str| {
            format!(
                "{} {}, {}({})",
                name,
                Register(self.t_register()),
                self.immediate(),
                Register(self.s_register())
            )
        };
        let dtsh = |name: &str| {
            format!(
                "{} {}, {}, {}",
                name,
                Register(self.d_register()),
                Register(self.t_register()),
                self.shift_amount()
            )
        };
        let dts = |name: &str| {
            format!(
                "{} {}, {}, {}",
                name,
                Register(self.d_register()),
                Register(self.t_register()),
                Register(self.s_register())
            )
        };
        let s = |name: &str| format!("{} {}", name, Register(self.s_register()));
        let st = |name: &str| {
            format!(
                "{} {}, {}",
                name,
                Register(self.s_register()),
                Register(self.t_register())
            )
        };
        let si = |name: &str| {
            format!(
                "{} {}, {}",
                name,
                Register(self.s_register()),
                self.immediate()
            )
        };
        let tsi = |name: &str| {
            format!(
                "{} {}, {}, {}",
                name,
                Register(self.t_register()),
                Register(self.s_register()),
                self.immediate()
            )
        };
        let tsiu = |name: &str| {
            format!(
                "{} {}, {}, 0x{:x}",
                name,
                Register(self.t_register()),
                Register(self.s_register()),
                self.immediate() as u16
            )
        };

        match self.op_code() {
            OP_R_TYPE => match self.function() {
                FUNCTION_SLL => {
                    if self.0 == 0 {
                        "noop".to_string()
                    } else {
                        dtsh("sll")
                    }
                }
                FUNCTION_SRL => dtsh("srl"),
                FUNCTION_SRA => dtsh("sra"),
                FUNCTION_SLLV => dts("sllv"),
                FUNCTION_SRLV => dts("srlv"),
                FUNCTION_SRAV => dts("srav"),
                FUNCTION_JR => s("jr"),
                FUNCTION_JALR => s("jalr"),
                FUNCTION_SYSCALL => "syscall".to_string(),
                FUNCTION_BREAK => "break".to_string(),
                FUNCTION_MFHI => format!("mfhi {}", Register(self.d_register())),
                FUNCTION_MTHI => s("mthi"),
                FUNCTION_MFLO => format!("mflo {}", Register(self.d_register())),
                FUNCTION_MTLO => s("mtlo"),
                FUNCTION_MULT => st("mult"),
                FUNCTION_MULTU => st("multu"),
                FUNCTION_DIV => st("div"),
                FUNCTION_DIVU => st("divu"),
                FUNCTION_ADD => dst("add"),
                FUNCTION_ADDU => dst("addu"),
                FUNCTION_SUB => dst("sub"),
                FUNCTION_SUBU => dst("subu"),
                FUNCTION_AND => dst("and"),
                FUNCTION_OR => dst("or"),
                FUNCTION_XOR => dst("xor"),
                FUNCTION_NOR => dst("nor"),
                FUNCTION_SLT => dst("slt"),
                FUNCTION_SLTU => dst("sltu"),
                function => panic!("Unknown R-type function 0x{:02x}", function),
            },
            OP_BCOND => match self.t_register() {
                BCOND_RT_BLTZ => si("bltz"),
                BCOND_RT_BGEZ => si("bgez"),
                BCOND_RT_BLTZAL => si("bltzal"),
                BCOND_RT_BGEZAL => si("bgezal"),
                rt => panic!("Unknown bcond operation, $rt = 0x{:02x}", rt),
            },
            OP_J => format!("j 0x{:x}", self.real_address(program_counter)),
            OP_JAL => format!("jal 0x{:x}", self.real_address(program_counter)),
            OP_BEQ => sti("beq"),
            OP_BNE => sti("bne"),
            OP_BLEZ => si("blez"),
            OP_BGTZ => si("bgtz"),
            OP_ADDI => tsi("addi"),
            OP_ADDIU => tsi("addiu"),
            OP_SLTI => tsi("slti"),
            OP_SLTIU => tsi("sltiu"),
            OP_ANDI => tsiu("andi"),
            OP_ORI => tsiu("ori"),
            OP_XORI => tsiu("xori"),
            OP_LUI => format!(
                "lui {}, 0x{:x}",
                Register(self.t_register()),
                self.immediate() as u16
            ),
            OP_LB => tis("lb"),
            OP_LH => tis("lh"),
            OP_LWL => tis("lwl"),
            OP_LW => tis("lw"),
            OP_LBU => tis("lbu"),
            OP_LHU => tis("lhu"),
            OP_LWR => tis("lwr"),
            OP_SB => tis("sb"),
            OP_SH => tis("sh"),
            OP_SWL => tis("swl"),
            OP_SW => tis("sw"),
            OP_SWR => tis("swr"),
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
