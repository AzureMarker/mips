//! Lower the IR to MIPS

use crate::ast::{ITypeOp, JTypeOp, RTypeOp};
use crate::ir::{IrInstruction, IrProgram};
use mips_types::constants::{
    FUNCTION_ADD, FUNCTION_ADDU, FUNCTION_AND, FUNCTION_JR, FUNCTION_OR, FUNCTION_SYSCALL, OP_ADDI,
    OP_ADDIU, OP_ANDI, OP_BEQ, OP_BGEZ_BLTZ, OP_BGTZ, OP_BLEZ, OP_BNE, OP_J, OP_JAL, OP_LUI, OP_LW,
    OP_ORI, OP_R_TYPE, OP_SLTI, OP_SW,
};
use mips_types::module::{
    R2KModule, R2KModuleHeader, R2KVersion, DATA_INDEX, R2K_MAGIC, SECTION_COUNT, TEXT_INDEX,
};

impl IrProgram {
    pub fn lower(self) -> R2KModule {
        let text: Vec<u8> = self
            .text
            .into_iter()
            .flat_map(|instruction| instruction.lower().to_be_bytes().to_vec())
            .collect();
        let mut section_sizes = [0; SECTION_COUNT];
        section_sizes[TEXT_INDEX] = text.len() as u32;
        section_sizes[DATA_INDEX] = self.data.len() as u32;

        R2KModule {
            header: R2KModuleHeader {
                magic: R2K_MAGIC,
                // TODO: Change to version 2 when we have the module name in the
                //       symbol & string tables
                version: R2KVersion::Version1,
                flags: 0, // Module flags are not used by R2K
                entry: 0, // Object modules do not specify an entry point
                section_sizes,
            },
            text_section: text,
            data_section: self.data,
            ..Default::default()
        }
    }
}

impl IrInstruction {
    pub fn lower(self) -> u32 {
        match self {
            IrInstruction::RType {
                op_code,
                rs,
                rt,
                rd,
                shift,
            } => {
                (OP_R_TYPE as u32) << 26
                    | (rs as u32) << 21
                    | (rt as u32) << 16
                    | (rd as u32) << 11
                    | (shift as u32) << 6
                    | op_code.function_code() as u32
            }
            IrInstruction::IType {
                op_code,
                rs,
                rt,
                immediate,
            } => {
                (op_code.code() as u32) << 26
                    | (rs as u32) << 21
                    | (rt as u32) << 16
                    | (immediate as u32 & 0xFFFF)
            }
            IrInstruction::JType {
                op_code,
                pseudo_address,
            } => (op_code.code() as u32) << 26 | pseudo_address,
            IrInstruction::Syscall => (OP_R_TYPE as u32) << 26 | FUNCTION_SYSCALL as u32,
        }
    }
}

impl RTypeOp {
    /// Get the MIPS function code for this R-type instruction
    pub fn function_code(&self) -> u8 {
        match self {
            RTypeOp::Add => FUNCTION_ADD,
            RTypeOp::Addu => FUNCTION_ADDU,
            RTypeOp::And => FUNCTION_AND,
            RTypeOp::Jr => FUNCTION_JR,
            RTypeOp::Or => FUNCTION_OR,
        }
    }
}

impl ITypeOp {
    /// Get the MIPS op code for this I-type instruction
    pub fn code(&self) -> u8 {
        match self {
            ITypeOp::Addi => OP_ADDI,
            ITypeOp::Addiu => OP_ADDIU,
            ITypeOp::Andi => OP_ANDI,
            ITypeOp::Beq => OP_BEQ,
            ITypeOp::Bne => OP_BNE,
            ITypeOp::Bgez | ITypeOp::Bgezal | ITypeOp::Bltz | ITypeOp::Bltzal => OP_BGEZ_BLTZ,
            ITypeOp::Bgtz => OP_BGTZ,
            ITypeOp::Blez => OP_BLEZ,
            ITypeOp::Lui => OP_LUI,
            ITypeOp::Lw => OP_LW,
            ITypeOp::Ori => OP_ORI,
            ITypeOp::Slti => OP_SLTI,
            ITypeOp::Sw => OP_SW,
        }
    }
}

impl JTypeOp {
    /// Get the MIPS op code for this I-type instruction
    pub fn code(&self) -> u8 {
        match self {
            JTypeOp::Jump => OP_J,
            JTypeOp::Jal => OP_JAL,
        }
    }
}
