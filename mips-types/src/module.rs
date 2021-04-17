use crate::constants::{DATA_OFFSET, SYM_DEF_LABEL, SYM_DEF_SEEN, SYM_MODE_MASK, TEXT_OFFSET};
use std::convert::{TryFrom, TryInto};
use std::io;
use std::io::{Read, Write};

pub const R2K_MAGIC: u16 = 0xFACE;
pub const SECTION_COUNT: usize = 10;
pub const TEXT_INDEX: usize = 0;
pub const RDATA_INDEX: usize = 1;
pub const DATA_INDEX: usize = 2;
pub const SDATA_INDEX: usize = 3;
pub const SBSS_INDEX: usize = 4;
pub const BSS_INDEX: usize = 5;
pub const RELOCATION_INDEX: usize = 6;
pub const REFERENCES_INDEX: usize = 7;
pub const SYMBOLS_INDEX: usize = 8;
pub const STRINGS_INDEX: usize = 9;

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
        let mut text_section = vec![0; header.section_sizes[TEXT_INDEX] as usize];
        let mut rdata_section = vec![0; header.section_sizes[RDATA_INDEX] as usize];
        let mut data_section = vec![0; header.section_sizes[DATA_INDEX] as usize];
        let mut sdata_section = vec![0; header.section_sizes[SDATA_INDEX] as usize];
        let sbss_size = header.section_sizes[SBSS_INDEX];
        let bss_size = header.section_sizes[BSS_INDEX];

        input.read_exact(&mut text_section)?;
        input.read_exact(&mut rdata_section)?;
        input.read_exact(&mut data_section)?;
        input.read_exact(&mut sdata_section)?;

        let relocation_section = (0..header.section_sizes[RELOCATION_INDEX])
            .map(|_| R2KRelocationEntry::parse(input))
            .collect::<Result<_, _>>()?;
        let reference_section = (0..header.section_sizes[REFERENCES_INDEX])
            .map(|_| R2KReferenceEntry::parse(input))
            .collect::<Result<_, _>>()?;
        let symbol_table = (0..header.section_sizes[SYMBOLS_INDEX])
            .map(|_| R2KSymbolEntry::parse(input))
            .collect::<Result<_, _>>()?;

        let mut string_table = vec![0; header.section_sizes[STRINGS_INDEX] as usize];
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

    /// Check if this module is a load module.
    /// Note: the file should also be executable on the file system
    pub fn is_load_module(&self) -> bool {
        self.header.entry != 0
    }

    /// Get a mutable reference and offset to the given section. If the section
    /// does not hold data (ex. undefined, bss, external) then None is returned.
    pub fn get_mut_section(&mut self, section: R2KSection) -> Option<(&mut [u8], u32)> {
        match section {
            R2KSection::Text => Some((&mut self.text_section, TEXT_OFFSET)),
            R2KSection::RData => Some((&mut self.rdata_section, DATA_OFFSET)),
            R2KSection::Data => Some((
                &mut self.data_section,
                DATA_OFFSET + self.rdata_section.len() as u32,
            )),
            R2KSection::SData => Some((
                &mut self.sdata_section,
                DATA_OFFSET + self.rdata_section.len() as u32 + self.data_section.len() as u32,
            )),
            R2KSection::Undefined
            | R2KSection::SBss
            | R2KSection::Bss
            | R2KSection::Absolute
            | R2KSection::External => None,
        }
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum R2KSection {
    Undefined,
    Text,
    RData,
    Data,
    SData,
    SBss,
    Bss,
    Absolute,
    External,
}

impl R2KSection {
    /// Parse the input as an R2K section number
    pub fn parse<R: Read>(input: &mut R) -> io::Result<Self> {
        read_u8(input)?
            .try_into()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Unknown section number"))
    }

    /// Write the section number
    pub fn write<W: Write>(&self, output: &mut W) -> io::Result<()> {
        output.write_all(&[*self as u8])
    }
}

impl TryFrom<u8> for R2KSection {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(R2KSection::Undefined),
            1 => Ok(R2KSection::Text),
            2 => Ok(R2KSection::RData),
            3 => Ok(R2KSection::Data),
            4 => Ok(R2KSection::SData),
            5 => Ok(R2KSection::SBss),
            6 => Ok(R2KSection::Bss),
            7 => Ok(R2KSection::Absolute),
            8 => Ok(R2KSection::External),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct R2KRelocationEntry {
    pub address: u32,
    pub section: R2KSection,
    pub rel_type: u8,
}

impl R2KRelocationEntry {
    /// Parse the input as an R2K relocation entry
    pub fn parse<R: Read>(input: &mut R) -> io::Result<Self> {
        let entry = Self {
            address: read_u32(input)?,
            section: R2KSection::parse(input)?,
            rel_type: read_u8(input)?,
        };

        // Skip past the two bytes of padding
        read_u8(input)?;
        read_u8(input)?;

        Ok(entry)
    }

    /// Write the entry
    pub fn write<W: Write>(&self, output: &mut W) -> io::Result<()> {
        // Note: there are two bytes of padding at the end of the binary format
        output.write_all(&self.address.to_be_bytes())?;
        self.section.write(output)?;
        output.write_all(&[self.rel_type, 0, 0])?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct R2KReferenceEntry {
    pub address: u32,
    pub str_idx: u32,
    pub section: R2KSection,
    pub ref_type: u8,
}

impl R2KReferenceEntry {
    /// Parse the input as an R2K reference entry
    pub fn parse<R: Read>(input: &mut R) -> io::Result<Self> {
        let entry = Self {
            address: read_u32(input)?,
            str_idx: read_u32(input)?,
            section: R2KSection::parse(input)?,
            ref_type: read_u8(input)?,
        };

        // Skip past the two bytes of padding
        read_u8(input)?;
        read_u8(input)?;

        Ok(entry)
    }

    /// Write the entry
    pub fn write<W: Write>(&self, output: &mut W) -> io::Result<()> {
        // Note: there are two bytes of padding at the end of the binary format
        output.write_all(&self.address.to_be_bytes())?;
        output.write_all(&self.str_idx.to_be_bytes())?;
        self.section.write(output)?;
        output.write_all(&[self.ref_type, 0, 0])?;

        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct R2KSymbolEntry {
    pub flags: u32,
    pub value: u32,
    pub str_idx: u32,
}

impl R2KSymbolEntry {
    /// Parse the input as an R2K symbol entry
    pub fn parse<R: Read>(input: &mut R) -> io::Result<Self> {
        Ok(Self {
            flags: read_u32(input)?,
            value: read_u32(input)?,
            str_idx: read_u32(input)?,
        })
    }

    /// Write the entry
    pub fn write<W: Write>(&self, output: &mut W) -> io::Result<()> {
        output.write_all(&self.flags.to_be_bytes())?;
        output.write_all(&self.value.to_be_bytes())?;
        output.write_all(&self.str_idx.to_be_bytes())?;

        Ok(())
    }

    /// Get the section where the symbol lives
    pub fn section(&self) -> R2KSection {
        R2KSection::try_from((self.flags & SYM_MODE_MASK) as u8)
            .expect("Symbol should have a valid section")
    }

    /// Check if this symbol represents a label
    pub fn is_label(&self) -> bool {
        self.flags & SYM_DEF_LABEL != 0
    }

    /// Check if this symbol represents the definition (local/export)
    pub fn has_definition(&self) -> bool {
        self.flags & SYM_DEF_SEEN != 0
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
