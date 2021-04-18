/// The config for `Processor`
#[derive(Debug)]
pub struct Config {
    /// Enables jump/branch delay slots
    pub enable_delay_slots: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            enable_delay_slots: false,
        }
    }
}
