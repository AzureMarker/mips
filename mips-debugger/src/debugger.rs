use mips_simulator::Processor;
use std::io;
use std::io::Write;

pub struct Debugger {
    pub processor: Processor,
    pub trace: bool,
}

impl Debugger {
    pub fn new(processor: Processor) -> Self {
        Self {
            processor,
            trace: false,
        }
    }

    /// Run the debugger
    pub fn run(&mut self) -> Result<(), io::Error> {
        loop {
            eprint!("mips-debugger> ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            self.run_command(&input);

            if !self.processor.running {
                break;
            }
        }

        info!("Program exited with code {}", self.processor.return_code);
        Ok(())
    }

    /// Run a command
    pub fn run_command(&mut self, command: &str) {
        let command: Vec<&str> = command.trim().split(' ').collect();

        match command.as_slice() {
            ["trace", enabled] => self.set_trace(*enabled),
            ["step"] | ["s"] => self.step(),
            ["exit"] | ["quit"] => self.processor.running = false,
            [""] => {} // Ignore empty input
            _ => eprintln!("Unknown input"),
        }
    }

    /// Execute the next instruction
    fn step(&mut self) {
        if self.trace {
            let instruction = self.processor.load_next_instruction();
            eprintln!("{}", instruction.stringify(self.processor.program_counter));
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
