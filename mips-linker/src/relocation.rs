use crate::util::{
    read_immediate, read_pseudo_address, read_word, write_immediate, write_pseudo_address,
    write_word,
};
use mips_types::constants::{REL_JUMP, REL_LOWER_IMM, REL_SPLIT_IMM, REL_UPPER_IMM, REL_WORD};
use mips_types::module::R2KRelocationEntry;

pub fn relocate(
    section: &mut [u8],
    section_num: u8,
    section_offset: u32,
    relocation: &mut Vec<R2KRelocationEntry>,
) {
    relocation.retain(|entry| {
        if entry.section != section_num {
            return true;
        }

        let address = entry.address as usize;

        match entry.rel_type {
            REL_LOWER_IMM => {
                let immediate = read_immediate(section, address);
                write_immediate(section, address, immediate + section_offset as u16);
            }
            REL_SPLIT_IMM => {
                let immediate = read_immediate(section, address);
                let second_immediate = read_immediate(section, address + 4);
                write_immediate(
                    section,
                    address,
                    second_immediate + (section_offset >> 16) as u16,
                );
                write_immediate(section, address + 4, immediate + section_offset as u16);
            }
            REL_WORD => {
                let word = read_word(section, address);
                write_word(section, address, word + section_offset);
            }
            REL_JUMP => {
                let pseudo_address = read_pseudo_address(section, address);
                let section_pseudo = (section_offset & 0x0FFFFFFC) >> 2;
                let new_pseudo_address = (pseudo_address + section_pseudo) & 0x03FFFFFF;
                write_pseudo_address(section, address, new_pseudo_address);
            }
            REL_UPPER_IMM => {
                let immediate = read_immediate(section, address);
                write_immediate(section, address, immediate + (section_offset >> 16) as u16);
            }
            _ => panic!("Unknown relocation type: {}", entry.rel_type),
        }

        false
    });
}
