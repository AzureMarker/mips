//! High-Level Intermediate Representation

use crate::ast::{ITypeOp, JTypeOp, RTypeOp};
use std::collections::HashMap;

#[derive(Debug)]
pub struct HirProgram {
    pub text: HirText,
    pub data: HirData,
    pub symbol_table: SymbolTable,
    pub globals: Vec<String>,
}

#[derive(Debug)]
pub struct HirText {
    pub instructions: Vec<HirInstruction>,
}

#[derive(Debug)]
pub enum HirInstruction {
    RType {
        op_code: RTypeOp,
        rs: u8,
        rt: u8,
        rd: u8,
        shift: u8,
    },
    IType {
        op_code: ITypeOp,
        rs: u8,
        rt: u8,
        immediate: i16,
    },
    JType {
        op_code: JTypeOp,
        label: String,
    },
    Syscall,
}

#[derive(Debug)]
pub struct HirData {
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct SymbolTable {
    pub map: HashMap<String, Symbol>,
}

#[derive(Debug)]
pub struct Symbol {
    pub location: SymbolLocation,
    pub offset: usize,
}

#[derive(Copy, Clone, Debug)]
pub enum SymbolLocation {
    Text,
    Data,
}
