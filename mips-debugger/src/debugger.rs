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
            ["print"] | ["p"] => self.print_registers(),
            ["exit"] | ["quit"] => self.processor.running = false,
            [""] => {} // Ignore empty input
            _ => eprintln!("Unknown input"),
        }
    }

    /// Execute the next instruction. Return if execution should continue.
    fn step(&mut self) -> bool {
        if self.trace {
            let instruction = self.processor.load_next_instruction();
            eprintln!(
                "0x{:08x}\t{}",
                self.processor.program_counter,
                instruction.stringify(self.processor.program_counter)
            );
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

    fn print_registers(&self) {
        println!(
            "Program Counter = 0x{:08x}\n\
             0  = $zero = 0x{:08x} 8  = $t0 = 0x{:08x} 16 = $s0 = 0x{:08x} 24 = $t8 = 0x{:08x}\n\
             1  = $at   = 0x{:08x} 9  = $t1 = 0x{:08x} 17 = $s1 = 0x{:08x} 25 = $t9 = 0x{:08x}\n\
             2  = $v0   = 0x{:08x} 10 = $t2 = 0x{:08x} 18 = $s2 = 0x{:08x} 26 = $k0 = 0x{:08x}\n\
             3  = $v1   = 0x{:08x} 11 = $t3 = 0x{:08x} 19 = $s3 = 0x{:08x} 27 = $k1 = 0x{:08x}\n\
             4  = $a0   = 0x{:08x} 12 = $t4 = 0x{:08x} 20 = $s4 = 0x{:08x} 28 = $gp = 0x{:08x}\n\
             5  = $a1   = 0x{:08x} 13 = $t5 = 0x{:08x} 21 = $s5 = 0x{:08x} 29 = $sp = 0x{:08x}\n\
             6  = $a2   = 0x{:08x} 14 = $t6 = 0x{:08x} 22 = $s6 = 0x{:08x} 30 = $fp = 0x{:08x}\n\
             7  = $a3   = 0x{:08x} 15 = $t7 = 0x{:08x} 23 = $s7 = 0x{:08x} 31 = $ra = 0x{:08x}",
            self.processor.program_counter,
            self.processor.registers.get(0),
            self.processor.registers.get(8),
            self.processor.registers.get(16),
            self.processor.registers.get(24),
            self.processor.registers.get(1),
            self.processor.registers.get(9),
            self.processor.registers.get(17),
            self.processor.registers.get(25),
            self.processor.registers.get(2),
            self.processor.registers.get(10),
            self.processor.registers.get(18),
            self.processor.registers.get(26),
            self.processor.registers.get(3),
            self.processor.registers.get(11),
            self.processor.registers.get(19),
            self.processor.registers.get(27),
            self.processor.registers.get(4),
            self.processor.registers.get(12),
            self.processor.registers.get(20),
            self.processor.registers.get(28),
            self.processor.registers.get(5),
            self.processor.registers.get(13),
            self.processor.registers.get(21),
            self.processor.registers.get(29),
            self.processor.registers.get(6),
            self.processor.registers.get(14),
            self.processor.registers.get(22),
            self.processor.registers.get(30),
            self.processor.registers.get(7),
            self.processor.registers.get(15),
            self.processor.registers.get(23),
            self.processor.registers.get(31),
        );
    }
}
