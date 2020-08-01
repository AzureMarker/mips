pub const OP_R_TYPE: u8 = 0;
pub const OP_JAL: u8 = 0x03;
pub const OP_BEQ: u8 = 0x04;
pub const OP_ADDI: u8 = 0x08;
pub const OP_ORI: u8 = 0x0D;
pub const OP_LUI: u8 = 0x0F;
pub const OP_LW: u8 = 0x23;
pub const OP_SW: u8 = 0x2B;

pub const FUNCTION_SLL: u8 = 0x00;
pub const FUNCTION_ADD: u8 = 0x20;
pub const FUNCTION_ADDU: u8 = 0x21;
pub const FUNCTION_BREAK: u8 = 0x0D;

/// The stack pointer register
pub const REG_SP: u8 = 29;
pub const REG_RA: u8 = 31;

/// The top of the stack
pub const STACK_START: u32 = 0x7fffe4d8;
pub const TEXT_OFFSET: u32 = 0x400000;
/// The read-only data segment offset
pub const R_DATA_OFFSET: u32 = 0x10000000;
