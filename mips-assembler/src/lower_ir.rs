//! Lower the IR to MIPS (in an R2K module)

use crate::ir::{IrProgram, RelocationEntry, RelocationType, Symbol, SymbolLocation, SymbolType};
use mips_types::constants::{
    REL_JUMP, REL_LOWER_IMM, REL_SPLIT_IMM, REL_UPPER_IMM, REL_WORD, SYM_DEF_LABEL, SYM_DEF_SEEN,
    SYM_DEF_UNDEF, SYM_GLOBAL,
};
use mips_types::module::{
    R2KModule, R2KModuleHeader, R2KRelocationEntry, R2KSymbolEntry, R2KVersion, DATA_INDEX,
    R2K_MAGIC, RDATA_INDEX, RELOCATION_INDEX, SDATA_INDEX, SECTION_COUNT, STRINGS_INDEX,
    SYMBOLS_INDEX, TEXT_INDEX,
};

impl IrProgram {
    pub fn lower(self) -> R2KModule {
        let text: Vec<u8> = self
            .text
            .into_iter()
            .flat_map(|instruction| instruction.lower().to_be_bytes().to_vec())
            .collect();
        let relocation: Vec<_> = self.relocation.iter().map(RelocationEntry::lower).collect();
        let symbols: Vec<_> = self.symbol_table.values().map(Symbol::lower).collect();
        let strings = self.string_table.as_bytes();
        let mut section_sizes = [0; SECTION_COUNT];

        section_sizes[TEXT_INDEX] = text.len() as u32;
        section_sizes[DATA_INDEX] = self.data.len() as u32;
        section_sizes[RDATA_INDEX] = self.rdata.len() as u32;
        section_sizes[SDATA_INDEX] = self.sdata.len() as u32;
        section_sizes[RELOCATION_INDEX] = relocation.len() as u32;
        section_sizes[SYMBOLS_INDEX] = symbols.len() as u32;
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
            symbol_table: symbols,
            string_table: strings,
            ..Default::default()
        }
    }
}

impl Symbol {
    fn lower(&self) -> R2KSymbolEntry {
        // Only label symbols are currently stored.
        let mut flags = SYM_DEF_LABEL;

        match self.ty {
            SymbolType::Local => {
                flags |= self.location.mode_flag() | SYM_DEF_SEEN;
            }
            SymbolType::Import => {
                flags |= SYM_DEF_UNDEF | SYM_GLOBAL;
            }
            SymbolType::Export => {
                flags |= self.location.mode_flag() | SYM_DEF_SEEN | SYM_GLOBAL;
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
    fn mode_flag(&self) -> u32 {
        match self {
            SymbolLocation::Text => 0x1,
            SymbolLocation::RData => 0x2,
            SymbolLocation::Data => 0x3,
            SymbolLocation::SData => 0x4,
        }
    }
}

impl RelocationEntry {
    fn lower(&self) -> R2KRelocationEntry {
        R2KRelocationEntry {
            address: self.offset as u32,
            section: self.location as u8,
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
