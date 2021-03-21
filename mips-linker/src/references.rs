use crate::util::{R2KStrings, R2KSymbolTable};
use mips_types::module::R2KReferenceEntry;

pub fn resolve_references(
    section: &mut [u8],
    section_num: u8,
    section_offset: u32,
    strings: R2KStrings,
    symbols: &R2KSymbolTable,
    references: &mut Vec<R2KReferenceEntry>,
) {
    references.retain(|reference| {
        if reference.section != section_num {
            return true;
        }

        let symbol_name = strings
            .get_str(reference.str_idx)
            .expect("Could not find string");
        let symbol = *symbols.get(symbol_name).expect("Could not find symbol");

        // todo

        false
    });
}
