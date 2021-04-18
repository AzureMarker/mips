/// Holds the processor's registers
#[derive(Debug)]
pub struct Registers {
    registers: [u32; 32],
    pub lo_register: u32,
    pub hi_register: u32,
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            registers: [0; 32],
            lo_register: 0,
            hi_register: 0,
        }
    }

    /// Get the value of a register
    pub fn get(&self, register: u8) -> u32 {
        if register == 0 {
            return 0;
        }

        self.registers[register as usize]
    }

    /// Set the value of a register
    pub fn set(&mut self, register: u8, value: u32) {
        if register == 0 {
            return;
        }

        self.registers[register as usize] = value
    }
}
