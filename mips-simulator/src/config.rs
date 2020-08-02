/// The config for `Processor`
#[derive(Debug)]
pub struct Config {
    /// Disables jump/branch delay slots
    pub disable_delay_slots: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            disable_delay_slots: false,
        }
    }
}
