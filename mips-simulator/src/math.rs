pub fn add_unsigned(a: u32, b: i32) -> u32 {
    if b < 0 {
        a.wrapping_sub((b * -1) as u32)
    } else {
        a.wrapping_add(b as u32)
    }
}
