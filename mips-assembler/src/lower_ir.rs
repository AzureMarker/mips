//! Lower the IR to MIPS (in an R2K module)

use crate::ir::{
    IrProgram, ReferenceEntry, ReferenceMethod, ReferenceTarget, ReferenceType, RelocationEntry,
    RelocationType, Symbol, SymbolLocation, SymbolType,
};
use mips_types::constants::{
    REF_METHOD_ADD, REF_METHOD_REPLACE, REF_METHOD_SUBTRACT, REF_TARGET_HALF_WORD, REF_TARGET_IMM,
    REF_TARGET_JUMP, REF_TARGET_SPLIT_IMM, REF_TARGET_WORD, REL_JUMP, REL_LOWER_IMM, REL_SPLIT_IMM,
    REL_UPPER_IMM, REL_WORD, SYM_DEF_LABEL, SYM_DEF_SEEN, SYM_GLOBAL,
};
use mips_types::module::{
    R2KModule, R2KModuleHeader, R2KReferenceEntry, R2KRelocationEntry, R2KSection, R2KSymbolEntry,
    R2KVersion, DATA_INDEX, R2K_MAGIC, RDATA_INDEX, REFERENCES_INDEX, RELOCATION_INDEX,
    SDATA_INDEX, SECTION_COUNT, STRINGS_INDEX, SYMBOLS_INDEX, TEXT_INDEX,
};
use std::array::IntoIter;
use std::convert::TryFrom;

impl IrProgram {
    pub fn lower(self) -> R2KModule {
        let text: Vec<u8> = self
            .text
            .into_iter()
            .flat_map(|instruction| IntoIter::new(instruction.lower().to_be_bytes()))
            .collect();
        let relocation = self.relocation.iter().map(RelocationEntry::lower).collect();
        let references = self.references.iter().map(ReferenceEntry::lower).collect();
        let symbols = self.symbol_table.values().map(Symbol::lower).collect();
        let strings = self.string_table.as_bytes();
        let mut section_sizes = [0; SECTION_COUNT];

        section_sizes[TEXT_INDEX] = text.len() as u32;
        section_sizes[DATA_INDEX] = self.data.len() as u32;
        section_sizes[RDATA_INDEX] = self.rdata.len() as u32;
        section_sizes[SDATA_INDEX] = self.sdata.len() as u32;
        section_sizes[RELOCATION_INDEX] = self.relocation.len() as u32;
        section_sizes[REFERENCES_INDEX] = self.references.len() as u32;
        section_sizes[SYMBOLS_INDEX] = self.symbol_table.len() as u32;
        section_sizes[STRINGS_INDEX] = strings.len() as u32;

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
            rdata_section: self.rdata,
            sdata_section: self.sdata,
            relocation_section: relocation,
            reference_section: references,
            symbol_table: symbols,
            string_table: strings,
            ..Default::default()
        }
    }
}

impl Symbol {
    fn lower(&self) -> R2KSymbolEntry {
        // Only label symbols are currently stored.
        let mut flags = 0;
        let section = self.location.lower();

        match self.ty {
            SymbolType::Local => {
                flags |= section as u32 | SYM_DEF_LABEL | SYM_DEF_SEEN;
            }
            SymbolType::Import => {
                flags |= R2KSection::External as u32 | SYM_GLOBAL;
            }
            SymbolType::Export => {
                flags |= section as u32 | SYM_DEF_LABEL | SYM_DEF_SEEN | SYM_GLOBAL;
            }
        }

        R2KSymbolEntry {
            flags,
            value: self.offset as u32,
            str_idx: self.string_offset as u32,
        }
    }
}

impl SymbolLocation {
    fn lower(&self) -> R2KSection {
        R2KSection::try_from(*self as u8).unwrap()
    }
}

impl RelocationEntry {
    fn lower(&self) -> R2KRelocationEntry {
        R2KRelocationEntry {
            address: self.offset as u32,
            section: self.location.lower(),
            rel_type: match self.relocation_type {
                RelocationType::LowerImmediate => REL_LOWER_IMM,
                RelocationType::UpperImmediate => REL_UPPER_IMM,
                RelocationType::SplitImmediate => REL_SPLIT_IMM,
                RelocationType::Word => REL_WORD,
                RelocationType::JumpAddress => REL_JUMP,
            },
        }
    }
}

impl ReferenceEntry {
    fn lower(&self) -> R2KReferenceEntry {
        R2KReferenceEntry {
            address: self.offset as u32,
            str_idx: self.str_idx as u32,
            section: self.location.lower(),
            ref_type: self.reference_type.lower(),
        }
    }
}

impl ReferenceType {
    fn lower(&self) -> u8 {
        let mut flags = 0;

        flags |= match self.method {
            ReferenceMethod::Add => REF_METHOD_ADD,
            ReferenceMethod::Replace => REF_METHOD_REPLACE,
            ReferenceMethod::Subtract => REF_METHOD_SUBTRACT,
        };

        flags |= match self.target {
            ReferenceTarget::Immediate => REF_TARGET_IMM,
            ReferenceTarget::HalfWord => REF_TARGET_HALF_WORD,
            ReferenceTarget::SplitImmediate => REF_TARGET_SPLIT_IMM,
            ReferenceTarget::Word => REF_TARGET_WORD,
            ReferenceTarget::JumpAddress => REF_TARGET_JUMP,
        };

        flags
    }
}
