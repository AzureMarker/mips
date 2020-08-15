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
        static REGISTERS: [&str; 32] = [
            "$zero", "$at", "$v0", "$v1", "$a0", "$a1", "$a2", "$a3", "$t0", "$t1", "$t2", "$t3",
            "$t4", "$t5", "$t6", "$t7", "$s0", "$s1", "$s2", "$s3", "$s4", "$s5", "$s6", "$s7",
            "$t8", "$t9", "$k0", "$k1", "$gp", "$sp", "$fp", "$ra",
        ];

        println!(
            "{} = 0x{:08x}",
            "PC".yellow(),
            self.processor.program_counter
        );

        for row in 0..8 {
            let col1 = row;
            let col2 = row + 8;
            let col3 = row + 16;
            let col4 = row + 24;

            println!(
                "{:2} = {:5} = 0x{:08x} {:2} = {:3} = 0x{:08x} {:2} = {:3} = 0x{:08x} {:2} = {:3} = 0x{:08x}",
                col1.to_string().blue(), REGISTERS[col1].yellow(), self.processor.registers.get(col1 as u8),
                col2.to_string().blue(), REGISTERS[col2].yellow(), self.processor.registers.get(col2 as u8),
                col3.to_string().blue(), REGISTERS[col3].yellow(), self.processor.registers.get(col3 as u8),
                col4.to_string().blue(), REGISTERS[col4].yellow(), self.processor.registers.get(col4 as u8)
            )
        }
    }
}
