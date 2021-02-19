//! Intermediate Representation

use crate::ast::{ITypeOp, JTypeOp, RTypeOp};
use std::collections::HashMap;

#[derive(Debug)]
pub struct IrProgram {
    pub text: Vec<IrInstruction>,
    pub data: Vec<u8>,
    pub rdata: Vec<u8>,
    pub sdata: Vec<u8>,
    pub symbol_table: HashMap<String, Symbol>,
    pub globals: Vec<String>,
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
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SymbolLocation {
    Text,
    Data,
    RData,
    SData,
}
