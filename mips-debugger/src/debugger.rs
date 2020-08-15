use mips_simulator::Processor;
use std::io;
use std::io::Write;

pub struct Debugger {
    processor: Processor,
    breakpoints: Vec<u32>,
    trace: bool,
}

impl Debugger {
    pub fn new(processor: Processor) -> Self {
        Self {
            processor,
            breakpoints: Vec::new(),
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
            ["trace", enabled] => self.set_trace(enabled),
            ["continue"] | ["c"] => self.continue_exec(),
            ["step"] | ["s"] => {
                self.step();
            }
            ["breakpoint", address] | ["b", address] => self.set_breakpoint(address),
            ["exit"] | ["quit"] => self.processor.running = false,
            [""] => {} // Ignore empty input
            _ => eprintln!("Unknown input"),
        }
    }

    /// Execute the next instruction. Return if execution should continue.
    fn step(&mut self) -> bool {
        if self.trace {
            let instruction = self.processor.load_next_instruction();
            eprintln!("{}", instruction.stringify(self.processor.program_counter));
        }

        self.processor.step();

        if self.breakpoints.contains(&self.processor.program_counter) {
            eprintln!("Breakpoint hit");
            false
        } else {
            self.processor.running
        }
    }

    /// Continue running until a breakpoint is hit or the program stops
    fn continue_exec(&mut self) {
        while self.step() {}
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

    fn set_breakpoint(&mut self, address: &str) {
        let address = address.strip_prefix("0x").unwrap_or(address);
        let address = match u32::from_str_radix(address, 16) {
            Ok(address) => address,
            Err(_) => {
                eprintln!("Invalid address");
                return;
            }
        };

        self.breakpoints.push(address);
    }
}
