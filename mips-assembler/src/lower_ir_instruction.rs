//! Lower IR instructions into MIPS

use crate::ast::{ITypeOp, JTypeOp, RTypeOp};
use crate::ir::IrInstruction;
use mips_types::constants::{
    FUNCTION_ADD, FUNCTION_ADDU, FUNCTION_AND, FUNCTION_DIV, FUNCTION_DIVU, FUNCTION_JALR,
    FUNCTION_JR, FUNCTION_MFHI, FUNCTION_MFLO, FUNCTION_MTHI, FUNCTION_MTLO, FUNCTION_MULT,
    FUNCTION_MULTU, FUNCTION_NOR, FUNCTION_OR, FUNCTION_SLL, FUNCTION_SLLV, FUNCTION_SLT,
    FUNCTION_SLTU, FUNCTION_SRA, FUNCTION_SRAV, FUNCTION_SRL, FUNCTION_SRLV, FUNCTION_SUB,
    FUNCTION_SUBU, FUNCTION_SYSCALL, FUNCTION_XOR, OP_ADDI, OP_ADDIU, OP_ANDI, OP_BCOND, OP_BEQ,
    OP_BGTZ, OP_BLEZ, OP_BNE, OP_J, OP_JAL, OP_LB, OP_LBU, OP_LH, OP_LHU, OP_LUI, OP_LW, OP_LWL,
    OP_LWR, OP_ORI, OP_R_TYPE, OP_SB, OP_SH, OP_SLTI, OP_SLTIU, OP_SW, OP_SWL, OP_SWR, OP_XORI,
};

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
            IrInstruction::Word(word) => word,
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
            RTypeOp::Div => FUNCTION_DIV,
            RTypeOp::Divu => FUNCTION_DIVU,
            RTypeOp::Jalr => FUNCTION_JALR,
            RTypeOp::Jr => FUNCTION_JR,
            RTypeOp::Mfhi => FUNCTION_MFHI,
            RTypeOp::Mflo => FUNCTION_MFLO,
            RTypeOp::Mthi => FUNCTION_MTHI,
            RTypeOp::Mtlo => FUNCTION_MTLO,
            RTypeOp::Mult => FUNCTION_MULT,
            RTypeOp::Multu => FUNCTION_MULTU,
            RTypeOp::Nor => FUNCTION_NOR,
            RTypeOp::Or => FUNCTION_OR,
            RTypeOp::Sll => FUNCTION_SLL,
            RTypeOp::Sllv => FUNCTION_SLLV,
            RTypeOp::Slt => FUNCTION_SLT,
            RTypeOp::Sltu => FUNCTION_SLTU,
            RTypeOp::Sra => FUNCTION_SRA,
            RTypeOp::Srav => FUNCTION_SRAV,
            RTypeOp::Srl => FUNCTION_SRL,
            RTypeOp::Srlv => FUNCTION_SRLV,
            RTypeOp::Sub => FUNCTION_SUB,
            RTypeOp::Subu => FUNCTION_SUBU,
            RTypeOp::Syscall => FUNCTION_SYSCALL,
            RTypeOp::Xor => FUNCTION_XOR,
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
            ITypeOp::Bcond => OP_BCOND,
            ITypeOp::Beq => OP_BEQ,
            ITypeOp::Bne => OP_BNE,
            ITypeOp::Bgtz => OP_BGTZ,
            ITypeOp::Blez => OP_BLEZ,
            ITypeOp::Lui => OP_LUI,
            ITypeOp::Lb => OP_LB,
            ITypeOp::Lbu => OP_LBU,
            ITypeOp::Lh => OP_LH,
            ITypeOp::Lhu => OP_LHU,
            ITypeOp::Lw => OP_LW,
            ITypeOp::Lwl => OP_LWL,
            ITypeOp::Lwr => OP_LWR,
            ITypeOp::Ori => OP_ORI,
            ITypeOp::Slti => OP_SLTI,
            ITypeOp::Sltiu => OP_SLTIU,
            ITypeOp::Sb => OP_SB,
            ITypeOp::Sh => OP_SH,
            ITypeOp::Sw => OP_SW,
            ITypeOp::Swl => OP_SWL,
            ITypeOp::Swr => OP_SWR,
            ITypeOp::Xori => OP_XORI,
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
