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
