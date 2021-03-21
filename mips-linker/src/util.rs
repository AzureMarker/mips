use mips_types::module::R2KSymbolEntry;
use std::collections::HashMap;
use std::str::from_utf8;

/// Read a word (u32) from the section at the given offset
pub fn read_word(section: &[u8], address: usize) -> u32 {
    let bytes = [
        section[address],
        section[address + 1],
        section[address + 2],
        section[address + 3],
    ];
    u32::from_be_bytes(bytes)
}

/// Update the immediate value of the instruction at the address.
/// The value is added to the immediate.
pub fn update_immediate(section: &mut [u8], address: usize, value: u16) {
    let immediate = read_word(section, address) as u16;
    let new_immediate = immediate + value;
    let new_bytes = new_immediate.to_be_bytes();
    section[(address + 2)..(address + 4)].copy_from_slice(&new_bytes);
}

/// Read the strings section into a map from offset to string
pub fn read_strings(strings: &[u8]) -> HashMap<usize, String> {
    let mut map = HashMap::new();
    let mut s = String::new();
    let mut index = 0;

    for (i, byte) in strings.iter().copied().enumerate() {
        if byte == 0 {
            map.insert(index, std::mem::take(&mut s));
            index = i + 1;
            continue;
        }

        s.push(byte as char);
    }

    map
}

#[derive(Copy, Clone)]
pub struct R2KStrings<'a> {
    inner: &'a [u8],
}

impl<'a> R2KStrings<'a> {
    pub fn new(strings: &'a [u8]) -> Self {
        Self { inner: strings }
    }

    pub fn get_str(&self, id: u32) -> Option<&str> {
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
