//! Lower the IR to MIPS (in an R2K module)

use crate::ir::{IrProgram, Symbol, SymbolLocation};
use mips_types::constants::{SYM_DEF_LABEL, SYM_DEF_SEEN, SYM_DEF_UNDEF};
use mips_types::module::{
    R2KModule, R2KModuleHeader, R2KSymbolEntry, R2KVersion, DATA_INDEX, R2K_MAGIC, RDATA_INDEX,
    SDATA_INDEX, SECTION_COUNT, STRINGS_INDEX, SYMBOLS_INDEX, TEXT_INDEX,
};

impl IrProgram {
    pub fn lower(self) -> R2KModule {
        let text: Vec<u8> = self
            .text
            .into_iter()
            .flat_map(|instruction| instruction.lower().to_be_bytes().to_vec())
            .collect();
        let mut symbols: Vec<_> = self.symbol_table.values().map(Symbol::lower).collect();
        let strings = self.string_table.as_bytes();
        let mut section_sizes = [0; SECTION_COUNT];

        // Add the globals imports into the symbol table
        for global in self.globals {
            if !self.symbol_table.contains_key(&global) {
                let str_idx = self
                    .string_table
                    .get_offset(&global)
                    .expect("Global import not found in string table")
                    as u32;
                symbols.push(R2KSymbolEntry {
                    flags: SYM_DEF_UNDEF,
                    value: 0,
                    str_idx,
                })
            }
        }

        section_sizes[TEXT_INDEX] = text.len() as u32;
        section_sizes[DATA_INDEX] = self.data.len() as u32;
        section_sizes[RDATA_INDEX] = self.rdata.len() as u32;
        section_sizes[SDATA_INDEX] = self.sdata.len() as u32;
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
            symbol_table: symbols,
            string_table: strings,
            ..Default::default()
        }
    }
}

impl Symbol {
    fn lower(&self) -> R2KSymbolEntry {
        R2KSymbolEntry {
            // Symbols are only stored in the IR symbol table if we've seen
            // them, and only label symbols are currently stored.
            flags: self.location.mode_flag() & SYM_DEF_LABEL & SYM_DEF_SEEN,
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
