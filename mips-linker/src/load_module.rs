//! Transform an object module into a load module

use crate::references::resolve_references;
use crate::relocation::relocate;
use crate::util::{make_symbol_table, R2KStrings};
use mips_types::constants::{R2K_ENTRYPOINT, TEXT_OFFSET};
use mips_types::module::{R2KModule, REFERENCES_INDEX, RELOCATION_INDEX};

pub fn obj_to_load_module(obj_module: &mut R2KModule) {
    relocate(obj_module);
    resolve_references(obj_module);

    let strings = R2KStrings::new(&obj_module.string_table);
    let symbols = make_symbol_table(strings, &obj_module.symbol_table);

    obj_module.header.section_sizes[RELOCATION_INDEX] = obj_module.relocation_section.len() as u32;
    obj_module.header.section_sizes[REFERENCES_INDEX] = obj_module.reference_section.len() as u32;

    if obj_module.reference_section.is_empty() {
        // All references are resolved, the output is a load module
        obj_module.header.entry = symbols
            .get(R2K_ENTRYPOINT)
            .map(|entry_symbol| TEXT_OFFSET + entry_symbol.value)
            .unwrap_or(TEXT_OFFSET);
    } else {
        // Not all references were resolved, the output is an object file
        obj_module.header.entry = 0;

        let missing_symbol_names: Vec<_> = obj_module
            .reference_section
            .iter()
            .map(|reference| {
                strings
                    .get_str(reference.str_idx)
                    .expect("Could not find string in string table")
            })
            .collect();

        log::info!(
            "Not all references were resolved. Missing {} symbol(s):",
            missing_symbol_names.len()
        );
        for symbol_name in missing_symbol_names {
            log::info!("  {}", symbol_name);
        }
    };
}
