use std::convert::{TryFrom, TryInto};
use std::io;
use std::io::{Read, Write};

pub const R2K_MAGIC: u16 = 0xFACE;
pub const SECTION_COUNT: usize = 10;

/// An R2K module
#[derive(Debug)]
pub struct R2KModule {
    pub header: R2KModuleHeader,
    pub sections: Vec<Vec<u8>>,
}

/// R2K's module header
#[derive(Debug)]
pub struct R2KModuleHeader {
    /// Must be `R2K_MAGIC`
    pub magic: u16,
    pub version: R2KVersion,
    pub flags: u32,
    pub entry: u32,
    pub section_sizes: Vec<u32>,
}

impl R2KModule {
    /// Parse the input as an R2K module
    pub fn parse<R: Read>(input: &mut R) -> io::Result<Self> {
        let header = R2KModuleHeader::parse(input)?;
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

    /// Write the module
    pub fn write<W: Write>(&self, output: &mut W) -> io::Result<()> {
        self.header.write(output)?;

        for section in &self.sections {
            output.write_all(&section)?;
        }

        Ok(())
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

impl R2KModuleHeader {
    /// Parse the input as an R2K module header
    pub fn parse<R: Read>(input: &mut R) -> io::Result<Self> {
        let magic = read_u16(input)?;

        if magic != R2K_MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid magic number",
            ));
        }

        Ok(Self {
            magic,
            version: read_u16(input)?.try_into().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "Unknown version number")
            })?,
            flags: read_u32(input)?,
            entry: read_u32(input)?,
            section_sizes: (0..SECTION_COUNT)
                .map(|_| read_u32(input))
                .collect::<io::Result<_>>()?,
        })
    }

    /// Write the module header
    pub fn write<W: Write>(&self, output: &mut W) -> io::Result<()> {
        output.write_all(&self.magic.to_be_bytes())?;
        output.write_all(&(self.version as u16).to_be_bytes())?;
        output.write_all(&self.flags.to_be_bytes())?;
        output.write_all(&self.entry.to_be_bytes())?;

        for section_size in &self.section_sizes {
            output.write_all(&section_size.to_be_bytes())?;
        }

        Ok(())
    }
}

#[repr(u16)]
#[derive(Copy, Clone, Debug)]
pub enum R2KVersion {
    Version1 = 0x0F22,
    Version2 = 0x18DC,
}

impl TryFrom<u16> for R2KVersion {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            v if v == R2KVersion::Version1 as u16 => Ok(R2KVersion::Version1),
            v if v == R2KVersion::Version2 as u16 => Ok(R2KVersion::Version2),
            _ => Err(()),
        }
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
