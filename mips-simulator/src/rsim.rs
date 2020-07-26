/// RSIM's module header
pub struct ModuleHeader {
    magic: u16,
    version: u16,
    flags: u32,
    entry: u32,
    section_sizes: [u32; 10],
}
