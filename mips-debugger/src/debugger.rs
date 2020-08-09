use mips_simulator::Processor;

pub struct Debugger {
    pub processor: Processor,
    pub trace: bool,
}

impl Debugger {
    /// Run a command. Returns true if execution should continue, else false.
    pub fn run_command(&mut self, command: &str) -> bool {
        let command: Vec<_> = command.trim().split(' ').collect();

        match command.as_slice() {
            ["trace", enabled] => match *enabled {
                "on" => {
                    self.trace = true;
                    eprintln!("Instruction tracing is ON");
                }
                "off" => {
                    self.trace = false;
                    eprintln!("Instruction tracing is OFF");
                }
                _ => eprintln!("Unknown input"),
            },
            ["step"] | ["s"] => {
                if self.trace {
                    eprintln!("{:?}", self.processor.load_next_instruction());
                }

                self.processor.step();

                if !self.processor.running {
                    return false;
                }
            }
            ["exit"] => return false,
            _ => eprintln!("Unknown input"),
        }

        true
    }
}
