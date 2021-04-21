use crate::util::{
    read_immediate, read_pseudo_address, read_word, write_immediate, write_pseudo_address,
    write_word,
};
use mips_types::constants::{REL_JUMP, REL_LOWER_IMM, REL_SPLIT_IMM, REL_UPPER_IMM, REL_WORD};
use mips_types::module::{R2KModule, R2KRelocationEntry};

pub fn relocate(obj_module: &mut R2KModule) {
    let mut relocation = std::mem::take(&mut obj_module.relocation_section);

    relocation.retain(|entry| {
        let section_offset = obj_module.get_section_offset(entry.section).unwrap_or(0);
        let section_data = match obj_module.get_mut_section(entry.section) {
            Some(data) => data,
            None => return true,
        };

        add_to_relocation_value(entry, section_data, section_offset);
        false
    });

    obj_module.relocation_section = relocation;
}

/// Update the value stored in the section by adding the new value to it.
pub fn add_to_relocation_value(entry: &R2KRelocationEntry, section_data: &mut [u8], value: u32) {
    let address = entry.address as usize;

    match entry.rel_type {
        REL_LOWER_IMM => {
            let immediate = read_immediate(section_data, address);
            write_immediate(section_data, address, immediate + value as u16);
        }
        REL_SPLIT_IMM => {
            let immediate = read_immediate(section_data, address);
            let second_immediate = read_immediate(section_data, address + 4);
            write_immediate(section_data, address, immediate + (value >> 16) as u16);
            write_immediate(section_data, address + 4, second_immediate + value as u16);
        }
        REL_WORD => {
            let word = read_word(section_data, address);
            write_word(section_data, address, word + value);
        }
        REL_JUMP => {
            let pseudo_address = read_pseudo_address(section_data, address);
            let section_pseudo = (value & 0x0FFFFFFC) >> 2;
            let new_pseudo_address = (pseudo_address + section_pseudo) & 0x03FFFFFF;
            write_pseudo_address(section_data, address, new_pseudo_address);
        }
        REL_UPPER_IMM => {
            let immediate = read_immediate(section_data, address);
            write_immediate(section_data, address, immediate + (value >> 16) as u16);
        }
        _ => panic!("Unknown relocation type: {}", entry.rel_type),
    }
}
