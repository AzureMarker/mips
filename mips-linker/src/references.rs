use crate::util::{
    make_symbol_table, read_half, read_immediate, read_pseudo_address, read_word, write_half,
    write_immediate, write_pseudo_address, write_word, R2KStrings,
};
use mips_types::constants::{
    REF_METHOD_ADD, REF_METHOD_MASK, REF_METHOD_REPLACE, REF_METHOD_SUBTRACT, REF_TARGET_HALF_WORD,
    REF_TARGET_IMM, REF_TARGET_JUMP, REF_TARGET_MASK, REF_TARGET_SPLIT_IMM, REF_TARGET_WORD,
};
use mips_types::module::{R2KModule, R2KSection};
use std::collections::HashMap;
use std::ops::{Add, Sub};

pub fn resolve_references(obj_module: &mut R2KModule) {
    let strings = R2KStrings::new(&obj_module.string_table);
    let symbols: HashMap<_, _> = make_symbol_table(strings, &obj_module.symbol_table)
        .into_iter()
        .map(|(key, value)| (key.to_string(), *value))
        .collect();
    let mut references = std::mem::take(&mut obj_module.reference_section);

    references.retain(|reference| {
        let strings = R2KStrings::new(&obj_module.string_table);
        let symbol_name = strings
            .get_str(reference.str_idx)
            .expect("Could not find string");
        let symbol = *symbols.get(symbol_name).expect("Could not find symbol");
        let address = reference.address as usize;
        let symbol_section_offset = obj_module.get_section_offset(symbol.section()).unwrap_or(0);
        let symbol_value = match symbol.section() {
            R2KSection::Undefined | R2KSection::Absolute => symbol.value,
            R2KSection::Text | R2KSection::RData | R2KSection::Data | R2KSection::SData => {
                symbol.value + symbol_section_offset
            }
            R2KSection::SBss | R2KSection::Bss => {
                unimplemented!()
            }
            R2KSection::External => {
                log::info!(
                    "Could not find symbol '{}' when resolving references",
                    symbol_name
                );
                return true;
            }
        };

        let section_data = match obj_module.get_mut_section(reference.section) {
            Some(res) => res,
            None => return true,
        };

        let method = reference.ref_type & REF_METHOD_MASK;
        let target = reference.ref_type & REF_TARGET_MASK;

        match target {
            REF_TARGET_IMM => {
                let immediate = read_immediate(section_data, address);
                let new_immediate = apply_method(method, symbol_value as u16, immediate);
                write_immediate(section_data, address, new_immediate);
            }
            REF_TARGET_HALF_WORD => {
                let half = read_half(section_data, address);
                let new_half = apply_method(method, symbol_value as u16, half);
                write_half(section_data, address, new_half);
            }
            REF_TARGET_SPLIT_IMM => {
                let immediate = read_immediate(section_data, address);
                let second_immediate = read_immediate(section_data, address + 4);
                let new_immediate = apply_method(method, (symbol_value >> 16) as u16, immediate);
                let new_second_immediate =
                    apply_method(method, symbol_value as u16, second_immediate);

                write_immediate(section_data, address, new_immediate);
                write_immediate(section_data, address + 4, new_second_immediate);
            }
            REF_TARGET_WORD => {
                let word = read_word(section_data, address);
                let new_word = apply_method(method, symbol_value, word);
                write_word(section_data, address, new_word);
            }
            REF_TARGET_JUMP => {
                let pseudo_address = read_pseudo_address(section_data, address);
                let new_pseudo_address =
                    apply_method(method, (symbol_value & 0x0FFFFFFC) >> 2, pseudo_address);
                write_pseudo_address(section_data, address, new_pseudo_address);
            }
            _ => panic!("Unknown target: 0x{:02x}", target),
        }
        false
    });

    obj_module.reference_section = references;
}

fn apply_method<T: Add<Output = T> + Sub<Output = T>>(
    method: u8,
    symbol_value: T,
    existing_value: T,
) -> T {
    match method {
        REF_METHOD_ADD => existing_value + symbol_value,
        REF_METHOD_REPLACE => symbol_value,
        REF_METHOD_SUBTRACT => symbol_value - existing_value,
        _ => panic!("Unknown method: 0x{:02x}", method),
    }
}
