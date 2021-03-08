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
    pub relocation: Vec<RelocationEntry>,
    pub references: Vec<ReferenceEntry>,
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
    RData,
    Data,
    SData,
}

impl SymbolLocation {
    pub fn section_number(&self) -> u8 {
        // Zero represents the undefined section, which we don't have
        *self as u8 + 1
    }
}

#[derive(Debug)]
pub enum SymbolType {
    Local,
    Import,
    Export,
}

#[derive(Debug)]
pub struct RelocationEntry {
    pub offset: usize,
    pub location: SymbolLocation,
    pub relocation_type: RelocationType,
}

impl RelocationEntry {
    pub fn split_immediate(offset: usize) -> Self {
        Self {
            offset,
            location: SymbolLocation::Text,
            relocation_type: RelocationType::SplitImmediate,
        }
    }

    pub fn jump(offset: usize) -> Self {
        Self {
            offset,
            location: SymbolLocation::Text,
            relocation_type: RelocationType::JumpAddress,
        }
    }
}

#[derive(Debug)]
pub enum RelocationType {
    /// Update the immediate field with the lower 16 bits of the section offset
    LowerImmediate,
    /// Update the immediate field with the lower 16 bits of the section offset
    UpperImmediate,
    /// Update the immediate field of the next two instructions with the upper
    /// (LUI) then lower (ORI) 16 bits of the section offset.
    SplitImmediate,
    /// Update the full word with the section offset
    Word,
    /// Update the jump instruction's pseudo address with the section offset
    JumpAddress,
}

#[derive(Debug)]
pub struct ReferenceEntry {
    pub offset: usize,
    pub str_idx: usize,
    pub location: SymbolLocation,
    pub reference_type: ReferenceType,
}

impl ReferenceEntry {
    pub fn split_immediate(symbol: &Symbol, offset: usize) -> Self {
        Self {
            offset,
            str_idx: symbol.string_offset,
            location: SymbolLocation::Text,
            reference_type: ReferenceType {
                method: ReferenceMethod::Replace,
                target: ReferenceTarget::SplitImmediate,
            },
        }
    }

    pub fn jump(symbol: &Symbol, offset: usize) -> Self {
        Self {
            offset,
            str_idx: symbol.string_offset,
            location: SymbolLocation::Text,
            reference_type: ReferenceType {
                method: ReferenceMethod::Replace,
                target: ReferenceTarget::JumpAddress,
            },
        }
    }
}

#[derive(Debug)]
pub struct ReferenceType {
    pub method: ReferenceMethod,
    pub target: ReferenceTarget,
}

#[derive(Debug)]
pub enum ReferenceMethod {
    Add,
    Replace,
    Subtract,
}

#[derive(Debug)]
pub enum ReferenceTarget {
    Immediate,
    HalfWord,
    SplitImmediate,
    Word,
    JumpAddress,
}
