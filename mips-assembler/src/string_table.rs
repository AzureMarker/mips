use std::collections::HashMap;

/// Builds an offset-based string table. Avoids string duplication by representing the string table
/// as a hashmap (deferring the translation to `Vec<u8>` until IR lowering).
#[derive(Default, Debug)]
pub struct StringTable {
    str_map: HashMap<String, usize>,
    next_offset: usize,
}

impl StringTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a string into the table and get it's offset. If the string was
    /// already inserted, the existing offset will be returned.
    pub fn insert(&mut self, s: String) -> usize {
        if let Some(offset) = self.str_map.get(&s) {
            return *offset;
        }

        let offset = self.next_offset;
        self.next_offset = offset + s.len() + 1;
        self.str_map.insert(s, offset);

        offset
    }

    /// Write the string table out as a contiguous block of bytes. The offsets
    /// obtained at insertion time can be used to recover the (null-terminated)
    /// strings.
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut strings: Vec<_> = self.str_map.iter().collect();
        let mut bytes = Vec::new();
        strings.sort_by_key(|(_, offset)| **offset);

        for (s, _) in strings {
            bytes.extend(s.bytes());
            bytes.push(0);
        }

        bytes
    }
}
