use mips_types::module::R2KSymbolEntry;
use std::collections::HashMap;
use std::str::from_utf8;

/// Read a word (u32) from the section
pub fn read_word(section: &[u8], address: usize) -> u32 {
    let bytes = [
        section[address],
        section[address + 1],
        section[address + 2],
        section[address + 3],
    ];
    u32::from_be_bytes(bytes)
}

/// Read a half (u16) from the section
pub fn read_half(section: &[u8], address: usize) -> u16 {
    let bytes = [section[address], section[address + 1]];
    u16::from_be_bytes(bytes)
}

/// Read the immediate field of an instruction
pub fn read_immediate(section: &[u8], address: usize) -> u16 {
    read_half(section, address + 2)
}

/// Read the pseudo address of an instruction
pub fn read_pseudo_address(section: &[u8], address: usize) -> u32 {
    let word = read_word(section, address);
    word & 0x03FFFFFF
}

/// Write a word (u32) to the section
pub fn write_word(section: &mut [u8], address: usize, value: u32) {
    let bytes = value.to_be_bytes();
    section[address..(address + 4)].copy_from_slice(&bytes);
}

/// Write a half (u16) to the section
pub fn write_half(section: &mut [u8], address: usize, value: u16) {
    let bytes = value.to_be_bytes();
    section[address..(address + 2)].copy_from_slice(&bytes);
}

/// Set the immediate value of an instruction
pub fn write_immediate(section: &mut [u8], address: usize, value: u16) {
    let new_bytes = value.to_be_bytes();
    section[(address + 2)..(address + 4)].copy_from_slice(&new_bytes);
}

/// Set the pseudo address of an instruction
pub fn write_pseudo_address(section: &mut [u8], address: usize, value: u32) {
    let bytes = value.to_be_bytes();
    section[address] = (section[address] & 0b11111100) + (bytes[0] & 0b00000011);
    section[(address + 1)..(address + 4)].copy_from_slice(&bytes[1..]);
}

#[derive(Copy, Clone)]
pub struct R2KStrings<'a> {
    inner: &'a [u8],
}

impl<'a> R2KStrings<'a> {
    pub fn new(strings: &'a [u8]) -> Self {
        Self { inner: strings }
    }

    pub fn get_str(&self, id: u32) -> Option<&'a str> {
        let id = id as usize;

        if id >= self.inner.len() {
            return None;
        }

        // Find ending null
        let len = self.inner[id..].iter().position(|byte| *byte == 0)?;

        from_utf8(&self.inner[id..(id + len)]).ok()
    }
}

pub type R2KSymbolTable<'a> = HashMap<&'a str, &'a R2KSymbolEntry>;

pub fn make_symbol_table<'a>(
    strings: R2KStrings<'a>,
    symbols: &'a [R2KSymbolEntry],
) -> R2KSymbolTable<'a> {
    let mut table = HashMap::new();

    for symbol in symbols {
        let string = strings
            .get_str(symbol.str_idx)
            .expect("Could not find symbol string in strings table");

        table.insert(string, symbol);
    }

    table
}
