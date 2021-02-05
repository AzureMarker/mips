/// Holds the processor's registers
#[derive(Debug)]
pub struct Registers([u32; 32]);

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}

impl Registers {
    pub fn new() -> Self {
        Registers([0; 32])
    }

    /// Get the value of a register
    pub fn get(&self, register: u8) -> u32 {
        if register == 0 {
            return 0;
        }

        self.0[register as usize]
    }

    /// Set the value of a register
    pub fn set(&mut self, register: u8, value: u32) {
        if register == 0 {
            return;
        }

        self.0[register as usize] = value
    }
}
