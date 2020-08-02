#[derive(Debug)]
pub struct Config {
    pub disable_delay_slots: bool
}

impl Default for Config {
    fn default() -> Self {
        Config {
            disable_delay_slots: false
        }
    }
}