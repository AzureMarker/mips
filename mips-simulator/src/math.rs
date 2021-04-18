/// Add unsigned and signed 32-bit numbers. Overflows will wrap.
pub fn add_unsigned(a: u32, b: i32) -> u32 {
    if b < 0 {
        a.wrapping_sub(-b as u32)
    } else {
        a.wrapping_add(b as u32)
    }
}
