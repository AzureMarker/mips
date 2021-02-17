// Operation codes
pub const OP_R_TYPE: u8 = 0;
pub const OP_BGEZ_BLTZ: u8 = 0x01;
pub const OP_J: u8 = 0x02;
pub const OP_JAL: u8 = 0x03;
pub const OP_BEQ: u8 = 0x04;
pub const OP_BNE: u8 = 0x05;
pub const OP_BLEZ: u8 = 0x06;
pub const OP_BGTZ: u8 = 0x07;
pub const OP_ADDI: u8 = 0x08;
pub const OP_ADDIU: u8 = 0x09;
pub const OP_SLTI: u8 = 0x0A;
pub const OP_SLTIU: u8 = 0x0B;
pub const OP_ANDI: u8 = 0x0C;
pub const OP_ORI: u8 = 0x0D;
pub const OP_XORI: u8 = 0x0E;
pub const OP_LUI: u8 = 0x0F;
pub const OP_LB: u8 = 0x20;
pub const OP_LW: u8 = 0x23;
pub const OP_SB: u8 = 0x28;
pub const OP_SW: u8 = 0x2B;

// R-type function codes
pub const FUNCTION_SLL: u8 = 0x00;
pub const FUNCTION_SRL: u8 = 0x02;
pub const FUNCTION_SRA: u8 = 0x03;
pub const FUNCTION_SLLV: u8 = 0x04;
pub const FUNCTION_SRLV: u8 = 0x06;
pub const FUNCTION_JR: u8 = 0x08;
pub const FUNCTION_SYSCALL: u8 = 0x0C;
pub const FUNCTION_BREAK: u8 = 0x0D;
pub const FUNCTION_MFHI: u8 = 0x10;
pub const FUNCTION_MFLO: u8 = 0x12;
pub const FUNCTION_MULT: u8 = 0x18;
pub const FUNCTION_MULTU: u8 = 0x19;
pub const FUNCTION_DIV: u8 = 0x1A;
pub const FUNCTION_DIVU: u8 = 0x1B;
pub const FUNCTION_ADD: u8 = 0x20;
pub const FUNCTION_ADDU: u8 = 0x21;
pub const FUNCTION_SUB: u8 = 0x22;
pub const FUNCTION_SUBU: u8 = 0x23;
pub const FUNCTION_AND: u8 = 0x24;
pub const FUNCTION_OR: u8 = 0x25;
pub const FUNCTION_XOR: u8 = 0x26;
pub const FUNCTION_SLT: u8 = 0x2A;
pub const FUNCTION_SLTU: u8 = 0x2B;

// Register numbers
pub const REG_V0: u8 = 2;
pub const REG_A0: u8 = 4;
/// The stack pointer register
pub const REG_SP: u8 = 29;
pub const REG_RA: u8 = 31;

pub static REGISTER_NAMES: [&str; 32] = [
    "$zero", "$at", "$v0", "$v1", "$a0", "$a1", "$a2", "$a3", "$t0", "$t1", "$t2", "$t3", "$t4",
    "$t5", "$t6", "$t7", "$s0", "$s1", "$s2", "$s3", "$s4", "$s5", "$s6", "$s7", "$t8", "$t9",
    "$k0", "$k1", "$gp", "$sp", "$fp", "$ra",
];

// Syscall codes
pub const SYSCALL_PRINT_INT: u32 = 1;
pub const SYSCALL_PRINT_STR: u32 = 4;
pub const SYSCALL_READ_INT: u32 = 5;
pub const SYSCALL_EXIT2: u32 = 17;

// Memory offsets
/// The bottom of the stack
pub const STACK_BOTTOM: u32 = 0x7fffefff;
pub const TEXT_OFFSET: u32 = 0x00400000;
pub const DATA_OFFSET: u32 = 0x10000000;
