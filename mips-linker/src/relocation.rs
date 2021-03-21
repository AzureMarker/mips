use crate::util::{read_word, update_immediate};
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
                update_immediate(section, address, section_offset as u16);
            }
            REL_SPLIT_IMM => {
                update_immediate(section, address, section_offset as u16);
                update_immediate(section, address + 4, (section_offset >> 16) as u16);
            }
            REL_WORD => {
                let word = read_word(section, address);
                let new_word = word + section_offset;
                let new_bytes = new_word.to_be_bytes();
                section[address..(address + 4)].copy_from_slice(&new_bytes);
            }
            REL_JUMP => {
                let word = read_word(section, address);
                let pseudo_address = word & 0x03FFFFFF;
                let section_pseudo = (section_offset & 0x0FFFFFFC) >> 2;
                let new_pseudo_address = (pseudo_address + section_pseudo) & 0x03FFFFFF;
                let bytes = new_pseudo_address.to_be_bytes();
                section[address] = (section[address] & 0b11111100) + (bytes[0]);
                section[(address + 1)..(address + 4)].copy_from_slice(&bytes[1..]);
            }
            REL_UPPER_IMM => {
                update_immediate(section, address, (section_offset >> 16) as u16);
            }
            _ => panic!("Unknown relocation type: {}", entry.rel_type),
        }

        false
    });
}
