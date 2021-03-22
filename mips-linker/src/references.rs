use crate::util::{
    read_half, read_immediate, read_pseudo_address, read_word, write_half, write_immediate,
    write_pseudo_address, write_word, R2KStrings, R2KSymbolTable,
};
use mips_types::constants::{
    REF_METHOD_ADD, REF_METHOD_MASK, REF_METHOD_REPLACE, REF_METHOD_SUBTRACT, REF_RESOLVED,
    REF_TARGET_HALF_WORD, REF_TARGET_IMM, REF_TARGET_JUMP, REF_TARGET_MASK, REF_TARGET_SPLIT_IMM,
    REF_TARGET_WORD, SYM_MODE_MASK,
};
use mips_types::module::{R2KReferenceEntry, R2KSymbolEntry};
use std::ops::{Add, Sub};

pub fn resolve_references(
    section: &mut [u8],
    section_num: u8,
    strings: R2KStrings,
    symbols: &R2KSymbolTable,
    references: &mut [R2KReferenceEntry],
) {
    for reference in references.iter_mut() {
        if reference.section != section_num /*|| !reference.is_resolvable()*/ || reference.is_resolved()
        {
            continue;
        }

        let address = reference.address as usize;
        let symbol_name = strings
            .get_str(reference.str_idx)
            .expect("Could not find string");
        let symbol = *symbols.get(symbol_name).expect("Could not find symbol");

        let method = reference.ref_type & REF_METHOD_MASK;
        let target = reference.ref_type & REF_TARGET_MASK;

        match target {
            REF_TARGET_IMM => {
                let immediate = read_immediate(section, address);
                let new_immediate = apply_method(method, symbol.value as u16, immediate);
                write_immediate(section, address, new_immediate);
            }
            REF_TARGET_HALF_WORD => {
                let half = read_half(section, address);
                let new_half = apply_method(method, symbol.value as u16, half);
                write_half(section, address, new_half);
            }
            REF_TARGET_SPLIT_IMM => {
                let immediate = read_immediate(section, address);
                let second_immediate = read_immediate(section, address + 4);
                let new_immediate = apply_method(method, (symbol.value >> 16) as u16, immediate);
                let new_second_immediate =
                    apply_method(method, symbol.value as u16, second_immediate);

                write_immediate(section, address, new_immediate);
                write_immediate(section, address + 4, new_second_immediate);
            }
            REF_TARGET_WORD => {
                let word = read_word(section, address);
                let new_word = apply_method(method, symbol.value, word);
                write_word(section, address, new_word);
            }
            REF_TARGET_JUMP => {
                let pseudo_address = read_pseudo_address(section, address);
                let new_pseudo_address =
                    apply_method(method, (symbol.value & 0x0FFFFFFC) >> 2, pseudo_address);
                write_pseudo_address(section, address, new_pseudo_address);
            }
            _ => panic!("Unknown target: 0x{:02x}", target),
        }

        reference.ref_type |= REF_RESOLVED;
    }
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
