//! Intermediate Representation

use crate::ast::{ITypeOp, JTypeOp, RTypeOp};
use crate::string_table::StringTable;
use std::collections::HashMap;

#[derive(Debug)]
pub struct IrProgram {
    pub text: Vec<IrInstruction>,
    pub data: Vec<u8>,
    pub rdata: Vec<u8>,
    pub sdata: Vec<u8>,
    pub symbol_table: HashMap<String, Symbol>,
    pub string_table: StringTable,
}

#[derive(Debug)]
pub enum IrInstruction {
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
        pseudo_address: u32,
    },
    Word(u32),
}

#[derive(Debug)]
pub struct Symbol {
    pub location: SymbolLocation,
    pub offset: usize,
    pub string_offset: usize,
    pub ty: SymbolType,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SymbolLocation {
    Text,
    Data,
    RData,
    SData,
}

#[derive(Debug)]
pub enum SymbolType {
    Local,
    Import,
    Export,
}
