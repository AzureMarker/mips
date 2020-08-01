pub const OP_R_TYPE: u8 = 0;
pub const OP_ORI: u8 = 0x0D;

pub const FUNCTION_ADD: u8 = 0x20;
pub const FUNCTION_BREAK: u8 = 0x0D;

/// The stack pointer register
pub const REG_SP: u8 = 27;

/// The top of the stack
pub const STACK_START: u32 = 0x7fffe4d8;
pub const TEXT_OFFSET: u32 = 0x400000;
