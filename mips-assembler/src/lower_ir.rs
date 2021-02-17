//! Lower the IR to MIPS (in an R2K module)

use crate::ir::IrProgram;
use mips_types::module::{
    R2KModule, R2KModuleHeader, R2KVersion, DATA_INDEX, R2K_MAGIC, SECTION_COUNT, TEXT_INDEX,
};

impl IrProgram {
    pub fn lower(self) -> R2KModule {
        let text: Vec<u8> = self
            .text
            .into_iter()
            .flat_map(|instruction| instruction.lower().to_be_bytes().to_vec())
            .collect();
        let mut section_sizes = [0; SECTION_COUNT];
        section_sizes[TEXT_INDEX] = text.len() as u32;
        section_sizes[DATA_INDEX] = self.data.len() as u32;

        R2KModule {
            header: R2KModuleHeader {
                magic: R2K_MAGIC,
                // TODO: Change to version 2 when we have the module name in the
                //       symbol & string tables
                version: R2KVersion::Version1,
                flags: 0, // Module flags are not used by R2K
                entry: 0, // Object modules do not specify an entry point
                section_sizes,
            },
            text_section: text,
            data_section: self.data,
            ..Default::default()
        }
    }
}
