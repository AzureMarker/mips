use crate::util::R2KStrings;
use mips_types::module::{
    R2KModule, R2KModuleHeader, R2KSection, DATA_INDEX, RDATA_INDEX, REFERENCES_INDEX,
    RELOCATION_INDEX, SDATA_INDEX, SECTION_COUNT, STRINGS_INDEX, SYMBOLS_INDEX, TEXT_INDEX,
};
use std::collections::HashSet;

/// Merge two object modules. The right module's sections will be placed after
/// the left module's sections.
pub fn merge_obj_modules(left: R2KModule, right: R2KModule) -> R2KModule {
    let left_sizes = left.header.section_sizes;
    let update_address = |section, address: &mut u32, entry_type: &str| match section {
        R2KSection::Text => *address += left_sizes[TEXT_INDEX],
        R2KSection::RData => *address += left_sizes[RDATA_INDEX],
        R2KSection::Data => *address += left_sizes[DATA_INDEX],
        R2KSection::SData => *address += left_sizes[SDATA_INDEX],
        _ => {
            log::warn!(
                "Unexpected section for {} entry during merge: {:?}",
                entry_type,
                section
            );
        }
    };

    // Merge text and data sections
    let mut merged_text = left.text_section;
    merged_text.extend(right.text_section);
    let mut merged_rdata = left.rdata_section;
    merged_rdata.extend(right.rdata_section);
    let mut merged_data = left.data_section;
    merged_data.extend(right.data_section);
    let mut merged_sdata = left.sdata_section;
    merged_sdata.extend(right.sdata_section);

    // Merge string table
    let mut merged_str_table = left.string_table;
    merged_str_table.extend(right.string_table);
    // TODO: deduplicate strings table? Maybe once all modules are merged.

    // Merge relocation
    let mut merged_relocation = left.relocation_section;
    merged_relocation.extend(right.relocation_section.into_iter().map(|mut entry| {
        update_address(entry.section, &mut entry.address, "relocation");
        entry
    }));

    // Merge references
    let mut merged_references = left.reference_section;
    merged_references.extend(right.reference_section.into_iter().map(|mut entry| {
        entry.str_idx += left_sizes[STRINGS_INDEX];
        update_address(entry.section, &mut entry.address, "reference");
        entry
    }));

    // Merge symbols
    let mut merged_symbols = left.symbol_table;
    merged_symbols.extend(right.symbol_table.into_iter().map(|mut symbol| {
        symbol.str_idx += left_sizes[STRINGS_INDEX];

        if symbol.is_label() {
            // Adjust the label offset
            update_address(symbol.section(), &mut symbol.value, "symbol");
        }

        symbol
    }));

    // Remove import symbols if the definition has been found
    let mut seen_defs = HashSet::new();
    let strings = R2KStrings::new(&merged_str_table);
    for symbol in &merged_symbols {
        if symbol.has_definition() {
            seen_defs.insert(strings.get_str(symbol.str_idx).unwrap());
        }
    }
    merged_symbols.retain(|symbol| {
        if symbol.has_definition() {
            // Always keep the local/export symbols
            return true;
        }

        // This symbol is of an import. Only keep it if we haven't seen the
        // symbol's definition.
        let symbol_str = strings.get_str(symbol.str_idx).unwrap();
        !seen_defs.contains(symbol_str)
    });

    let mut section_sizes = [0; SECTION_COUNT];
    section_sizes[TEXT_INDEX] = merged_text.len() as u32;
    section_sizes[DATA_INDEX] = merged_data.len() as u32;
    section_sizes[RDATA_INDEX] = merged_rdata.len() as u32;
    section_sizes[SDATA_INDEX] = merged_sdata.len() as u32;
    section_sizes[RELOCATION_INDEX] = merged_relocation.len() as u32;
    section_sizes[REFERENCES_INDEX] = merged_references.len() as u32;
    section_sizes[SYMBOLS_INDEX] = merged_symbols.len() as u32;
    section_sizes[STRINGS_INDEX] = merged_str_table.len() as u32;

    R2KModule {
        header: R2KModuleHeader {
            section_sizes,
            ..Default::default()
        },
        text_section: merged_text,
        rdata_section: merged_rdata,
        data_section: merged_data,
        sdata_section: merged_sdata,
        sbss_size: left.sbss_size + right.sbss_size,
        bss_size: left.bss_size + right.bss_size,
        relocation_section: merged_relocation,
        reference_section: merged_references,
        symbol_table: merged_symbols,
        string_table: merged_str_table,
    }
}
