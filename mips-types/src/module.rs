use std::convert::{TryFrom, TryInto};
use std::io;
use std::io::{Read, Write};

pub const R2K_MAGIC: u16 = 0xFACE;
pub const SECTION_COUNT: usize = 10;

/// An R2K module
#[derive(Debug)]
pub struct R2KModule {
    pub header: R2KModuleHeader,
    pub text_section: Vec<u8>,
    pub rdata_section: Vec<u8>,
    pub data_section: Vec<u8>,
    pub sdata_section: Vec<u8>,
    pub sbss_size: u32,
    pub bss_size: u32,
    // TODO: the below sections should be parsed
    pub relocation_section: Vec<u8>,
    pub reference_section: Vec<u8>,
    pub symbol_table: Vec<u8>,
    pub string_table: Vec<u8>,
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
        let mut text_section = vec![0; header.section_sizes[0] as usize];
        let mut rdata_section = vec![0; header.section_sizes[1] as usize];
        let mut data_section = vec![0; header.section_sizes[2] as usize];
        let mut sdata_section = vec![0; header.section_sizes[3] as usize];
        let sbss_size = header.section_sizes[4];
        let bss_size = header.section_sizes[5];

        input.read_exact(&mut text_section)?;
        input.read_exact(&mut rdata_section)?;
        input.read_exact(&mut data_section)?;
        input.read_exact(&mut sdata_section)?;

        Ok(Self {
            header,
            text_section,
            rdata_section,
            data_section,
            sdata_section,
            sbss_size,
            bss_size,
            // TODO: read these sections
            relocation_section: vec![],
            reference_section: vec![],
            symbol_table: vec![],
            string_table: vec![],
        })
    }

    /// Write the module
    pub fn write<W: Write>(&self, output: &mut W) -> io::Result<()> {
        self.header.write(output)?;

        output.write_all(&self.text_section)?;
        output.write_all(&self.rdata_section)?;
        output.write_all(&self.data_section)?;
        output.write_all(&self.sdata_section)?;

        // TODO: write out the remaining sections

        Ok(())
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
