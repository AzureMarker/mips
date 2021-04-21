use std::collections::HashMap;
use std::ffi::CString;
use std::fmt;
use std::fmt::{Debug, Formatter};

/// 1MB pages
const PAGE_SIZE: usize = 1024 * 1024;

/// An implementation of paged memory
/// TODO: Check for unaligned memory access
#[derive(Default)]
pub struct Memory {
    /// Pages indexed by first address in the page
    pages: HashMap<u32, [u8; PAGE_SIZE]>,
}

impl Memory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the byte at the address
    #[inline]
    pub fn get(&self, address: u32) -> u8 {
        let page_index = self.page_index(address);
        let address_offset = self.address_offset(address);

        if let Some(page) = self.pages.get(&page_index) {
            page[address_offset]
        } else {
            0
        }
    }

    /// Get the bytes in the address range
    pub fn get_range<const N: usize>(&self, address: u32) -> [u8; N] {
        let mut bytes = [0; N];

        for (i, byte) in bytes.iter_mut().enumerate() {
            *byte = self.get(address + i as u32);
        }

        bytes
    }

    /// Get the word (4 bytes) at the address
    pub fn get_word(&self, address: u32) -> u32 {
        let bytes = self.get_range(address);
        u32::from_be_bytes(bytes)
    }

    /// Get a null-terminated string starting from the address
    pub fn get_str(&self, address: u32) -> CString {
        let mut bytes = Vec::new();

        for i in address.. {
            let byte = self.get(i);

            if byte == 0 {
                break;
            }

            bytes.push(byte);
        }

        CString::new(bytes).unwrap()
    }

    /// Set a byte at the address
    #[inline(always)]
    pub fn set(&mut self, address: u32, value: u8) {
        let page_index = self.page_index(address);
        let address_offset = self.address_offset(address);
        let page = self
            .pages
            .entry(page_index)
            .or_insert_with(|| [0; PAGE_SIZE]);

        page[address_offset] = value;
    }

    /// Set a word (4 bytes) at the address
    #[inline(always)]
    pub fn set_word(&mut self, address: u32, value: u32) {
        for (i, byte) in value.to_be_bytes().iter().enumerate() {
            self.set(address + i as u32, *byte);
        }
    }

    /// Load some data into memory
    pub fn load_into_memory(&mut self, data: &[u8], offset: u32) {
        for (i, byte) in data.iter().enumerate() {
            self.set(offset + i as u32, *byte);
        }
    }

    /// Get the index of the page the address is in
    fn page_index(&self, address: u32) -> u32 {
        address - self.address_offset(address) as u32
    }

    /// Get the address's offset in its page
    fn address_offset(&self, address: u32) -> usize {
        address as usize % PAGE_SIZE
    }
}

impl Debug for Memory {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Memory({} pages)", self.pages.len())
    }
}
