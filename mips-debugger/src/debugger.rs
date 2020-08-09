use mips_simulator::Processor;

pub struct Debugger {
    pub processor: Processor,
    pub trace: bool,
}

impl Debugger {
    /// Run a command
    pub fn run_command(&mut self, command: &str) {
        let command: Vec<&str> = command.trim().split(' ').collect();

        match command.as_slice() {
            ["trace", enabled] => self.set_trace(*enabled),
            ["step"] | ["s"] => self.step(),
            ["exit"] => self.processor.running = false,
            _ => eprintln!("Unknown input"),
        }
    }

    /// Execute the next instruction
    fn step(&mut self) {
        if self.trace {
            eprintln!("{:?}", self.processor.load_next_instruction());
        }

        self.processor.step();
    }

    /// Set the trace option
    fn set_trace(&mut self, option: &str) {
        match option {
            "on" => {
                self.trace = true;
                eprintln!("Instruction tracing is ON");
            }
            "off" => {
                self.trace = false;
                eprintln!("Instruction tracing is OFF");
            }
            _ => eprintln!("Unknown input"),
        }
    }
}
