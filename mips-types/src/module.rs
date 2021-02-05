use std::convert::{TryFrom, TryInto};
use std::io;
use std::io::{Read, Write};

pub const R2K_MAGIC: u16 = 0xFACE;
pub const SECTION_COUNT: usize = 10;

/// An R2K module
#[derive(Debug, Default)]
pub struct R2KModule {
    pub header: R2KModuleHeader,
    pub text_section: Vec<u8>,
    pub rdata_section: Vec<u8>,
    pub data_section: Vec<u8>,
    pub sdata_section: Vec<u8>,
    pub sbss_size: u32,
    pub bss_size: u32,
    pub relocation_section: Vec<R2KRelocationEntry>,
    pub reference_section: Vec<R2KReferenceEntry>,
    pub symbol_table: Vec<R2KSymbolEntry>,
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
    pub section_sizes: [u32; SECTION_COUNT],
}

impl Default for R2KModuleHeader {
    fn default() -> Self {
        Self {
            magic: R2K_MAGIC,
            version: R2KVersion::Version1,
            flags: 0,
            entry: 0,
            section_sizes: [0; SECTION_COUNT],
        }
    }
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

        let relocation_section = (0..header.section_sizes[6])
            .map(|_| R2KRelocationEntry::parse(input))
            .collect::<Result<_, _>>()?;
        let reference_section = (0..header.section_sizes[7])
            .map(|_| R2KReferenceEntry::parse(input))
            .collect::<Result<_, _>>()?;
        let symbol_table = (0..header.section_sizes[8])
            .map(|_| R2KSymbolEntry::parse(input))
            .collect::<Result<_, _>>()?;

        let mut string_table = vec![0; header.section_sizes[9] as usize];
        input.read_exact(&mut string_table)?;

        Ok(Self {
            header,
            text_section,
            rdata_section,
            data_section,
            sdata_section,
            sbss_size,
            bss_size,
            relocation_section,
            reference_section,
            symbol_table,
            string_table,
        })
    }

    /// Write the module
    pub fn write<W: Write>(&self, output: &mut W) -> io::Result<()> {
        self.header.write(output)?;

        output.write_all(&self.text_section)?;
        output.write_all(&self.rdata_section)?;
        output.write_all(&self.data_section)?;
        output.write_all(&self.sdata_section)?;

        for entry in &self.relocation_section {
            entry.write(output)?;
        }

        for entry in &self.reference_section {
            entry.write(output)?;
        }

        for entry in &self.symbol_table {
            entry.write(output)?;
        }

        output.write_all(&self.string_table)?;

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
            section_sizes: {
                let mut sizes = [0; SECTION_COUNT];

                for entry in sizes.iter_mut() {
                    *entry = read_u32(input)?;
                }

                sizes
            },
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

#[derive(Debug)]
pub struct R2KRelocationEntry {
    pub address: u32,
    pub section: u8,
    pub rel_type: u8,
}

impl R2KRelocationEntry {
    /// Parse the input as an R2K relocation entry
    pub fn parse<R: Read>(input: &mut R) -> io::Result<Self> {
        Ok(Self {
            address: read_u32(input)?,
            section: read_u8(input)?,
            rel_type: read_u8(input)?,
        })
    }

    /// Write the entry
    pub fn write<W: Write>(&self, output: &mut W) -> io::Result<()> {
        output.write_all(&self.address.to_be_bytes())?;
        output.write_all(&[self.section, self.rel_type])?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct R2KReferenceEntry {
    pub address: u32,
    pub symbol: u32,
    pub section: u8,
    pub ref_type: u8,
}

impl R2KReferenceEntry {
    /// Parse the input as an R2K reference entry
    pub fn parse<R: Read>(input: &mut R) -> io::Result<Self> {
        Ok(Self {
            address: read_u32(input)?,
            symbol: read_u32(input)?,
            section: read_u8(input)?,
            ref_type: read_u8(input)?,
        })
    }

    /// Write the entry
    pub fn write<W: Write>(&self, output: &mut W) -> io::Result<()> {
        output.write_all(&self.address.to_be_bytes())?;
        output.write_all(&self.symbol.to_be_bytes())?;
        output.write_all(&[self.section, self.ref_type])?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct R2KSymbolEntry {
    pub flags: u32,
    pub value: u32,
    pub symbol: u32,
}

impl R2KSymbolEntry {
    /// Parse the input as an R2K symbol entry
    pub fn parse<R: Read>(input: &mut R) -> io::Result<Self> {
        Ok(Self {
            flags: read_u32(input)?,
            value: read_u32(input)?,
            symbol: read_u32(input)?,
        })
    }

    /// Write the entry
    pub fn write<W: Write>(&self, output: &mut W) -> io::Result<()> {
        output.write_all(&self.flags.to_be_bytes())?;
        output.write_all(&self.value.to_be_bytes())?;
        output.write_all(&self.symbol.to_be_bytes())?;

        Ok(())
    }
}

fn read_u8<R: Read>(input: &mut R) -> io::Result<u8> {
    let mut bytes = [0; 1];
    input.read_exact(&mut bytes)?;
    Ok(bytes[0])
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
