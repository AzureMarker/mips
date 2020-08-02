use std::io;
use std::io::Read;

const RSIM_MAGIC: u16 = 0xFACE;
const SECTION_COUNT: usize = 10;

/// An RSIM module
pub struct RsimModule {
    pub header: RsimModuleHeader,
    sections: Vec<Vec<u8>>,
}

/// RSIM's module header
#[derive(Debug)]
pub struct RsimModuleHeader {
    /// Must be `RSIM_MAGIC`
    magic: u16,
    pub version: u16,
    pub flags: u32,
    pub entry: u32,
    section_sizes: Vec<u32>,
}

impl RsimModule {
    /// Parse the input as an RSIM module
    pub fn parse<R: Read>(input: &mut R) -> io::Result<Self> {
        let header = RsimModuleHeader::parse(input)?;
        let mut sections = Vec::with_capacity(header.section_sizes.len());

        // Read each section's data
        // TODO: Handle the sections which don't use bytes as the length?
        //       "relocation, reference, and symbol table sizes are described as
        //       the number of entries rather than the number of bytes in the
        //       section."
        //       https://www.cs.rit.edu/~vcss345/documents/rlink.html#format
        for size in header.section_sizes.iter().copied() {
            let mut section = vec![0; size as usize];
            input.read_exact(&mut section)?;
            sections.push(section);
        }

        Ok(Self { header, sections })
    }

    /// Get the text section's data
    pub fn text_section(&self) -> &[u8] {
        &self.sections[0]
    }

    /// Get the rdata section's data
    pub fn read_only_data_section(&self) -> &[u8] {
        &self.sections[1]
    }

    /// Get the data section's data
    pub fn data_section(&self) -> &[u8] {
        &self.sections[2]
    }

    /// Get the sdata section's data
    pub fn small_data_section(&self) -> &[u8] {
        &self.sections[3]
    }
}

impl RsimModuleHeader {
    /// Parse the input as an RSIM module header
    pub fn parse<R: Read>(input: &mut R) -> io::Result<Self> {
        let magic = read_u16(input)?;

        if magic != RSIM_MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid magic number",
            ));
        }

        Ok(Self {
            magic,
            version: read_u16(input)?,
            flags: read_u32(input)?,
            entry: read_u32(input)?,
            section_sizes: (0..SECTION_COUNT)
                .into_iter()
                .map(|_| read_u32(input))
                .collect::<io::Result<_>>()?,
        })
    }
}

fn read_u16<R: Read>(input: &mut R) -> io::Result<u16> {
    let mut bytes = [0; 2];
    input.read_exact(&mut bytes)?;
    Ok(u16::from_be_bytes(bytes))
}

fn read_u32<R: Read>(input: &mut R) -> io::Result<u32> {
    let mut bytes = [0; 4];
    input.read_exact(&mut bytes)?;
    Ok(u32::from_be_bytes(bytes))
}
