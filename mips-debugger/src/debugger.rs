use colored::Colorize;
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
        #[rustfmt::skip]
        println!(
            "Program Counter = 0x{:08x}\n\
             {}  = {} = 0x{:08x} {}  = {} = 0x{:08x} {} = {} = 0x{:08x} {} = {} = 0x{:08x}\n\
             {}  = {}   = 0x{:08x} {}  = {} = 0x{:08x} {} = {} = 0x{:08x} {} = {} = 0x{:08x}\n\
             {}  = {}   = 0x{:08x} {} = {} = 0x{:08x} {} = {} = 0x{:08x} {} = {} = 0x{:08x}\n\
             {}  = {}   = 0x{:08x} {} = {} = 0x{:08x} {} = {} = 0x{:08x} {} = {} = 0x{:08x}\n\
             {}  = {}   = 0x{:08x} {} = {} = 0x{:08x} {} = {} = 0x{:08x} {} = {} = 0x{:08x}\n\
             {}  = {}   = 0x{:08x} {} = {} = 0x{:08x} {} = {} = 0x{:08x} {} = {} = 0x{:08x}\n\
             {}  = {}   = 0x{:08x} {} = {} = 0x{:08x} {} = {} = 0x{:08x} {} = {} = 0x{:08x}\n\
             {}  = {}   = 0x{:08x} {} = {} = 0x{:08x} {} = {} = 0x{:08x} {} = {} = 0x{:08x}",
            self.processor.program_counter,
            "0".blue(), "$zero".yellow(), self.processor.registers.get(0),
            "8".blue(), "$t0".yellow(), self.processor.registers.get(8),
            "16".blue(), "$s0".yellow(), self.processor.registers.get(16),
            "24".blue(), "$t8".yellow(), self.processor.registers.get(24),
            "1".blue(), "$at".yellow(), self.processor.registers.get(1),
            "9".blue(), "$t1".yellow(), self.processor.registers.get(9),
            "17".blue(), "$s1".yellow(), self.processor.registers.get(17),
            "25".blue(), "$t9".yellow(), self.processor.registers.get(25),
            "2".blue(), "$v0".yellow(), self.processor.registers.get(2),
            "10".blue(), "$t2".yellow(), self.processor.registers.get(10),
            "18".blue(), "$s2".yellow(), self.processor.registers.get(18),
            "26".blue(), "$k0".yellow(), self.processor.registers.get(26),
            "3".blue(), "$v1".yellow(), self.processor.registers.get(3),
            "11".blue(), "$t3".yellow(), self.processor.registers.get(11),
            "19".blue(), "$s3".yellow(), self.processor.registers.get(19),
            "27".blue(), "$k1".yellow(), self.processor.registers.get(27),
            "4".blue(), "$a0".yellow(), self.processor.registers.get(4),
            "12".blue(), "$t4".yellow(), self.processor.registers.get(12),
            "20".blue(), "$s4".yellow(), self.processor.registers.get(20),
            "28".blue(), "$gp".yellow(), self.processor.registers.get(28),
            "5".blue(), "$a1".yellow(), self.processor.registers.get(5),
            "13".blue(), "$t5".yellow(), self.processor.registers.get(13),
            "21".blue(), "$s5".yellow(), self.processor.registers.get(21),
            "29".blue(), "$sp".yellow(), self.processor.registers.get(29),
            "6".blue(), "$a2".yellow(), self.processor.registers.get(6),
            "14".blue(), "$t6".yellow(), self.processor.registers.get(14),
            "22".blue(), "$s6".yellow(), self.processor.registers.get(22),
            "30".blue(), "$fp".yellow(), self.processor.registers.get(30),
            "7".blue(), "$a3".yellow(), self.processor.registers.get(7),
            "15".blue(), "$t7".yellow(), self.processor.registers.get(15),
            "23".blue(), "$s7".yellow(), self.processor.registers.get(23),
            "31".blue(), "$ra".yellow(), self.processor.registers.get(31),
        );
    }
}
