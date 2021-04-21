use crate::Processor;
use mips_types::constants::*;
use std::io;
use std::io::{Read, Write};

impl Processor {
    /// Handle a syscall operation
    pub(crate) fn op_syscall(&mut self) {
        match self.registers.get(REG_V0) {
            SYSCALL_PRINT_INT => self.syscall_print_int(),
            SYSCALL_PRINT_STR => self.syscall_print_str(),
            SYSCALL_READ_INT => self.syscall_read_int(),
            SYSCALL_READ_STRING => self.syscall_read_str(),
            SYSCALL_EXIT2 => self.syscall_exit2(),
            operation => panic!("Unknown syscall operation {}", operation),
        }

        self.advance_program_counter();
    }

    /// Print an integer
    fn syscall_print_int(&mut self) {
        trace!("PRINT_INT");
        let value = self.registers.get(REG_A0) as i32;
        Self::print(value.to_string().as_bytes());
    }

    /// Print a string
    fn syscall_print_str(&mut self) {
        trace!("PRINT_STR");
        let str_address = self.registers.get(REG_A0);
        let input_str = self.memory.get_str(str_address);

        Self::print(input_str.as_bytes());
    }

    /// Read an integer from stdin
    fn syscall_read_int(&mut self) {
        trace!("READ_INT");
        let mut buffer = String::new();
        io::stdin()
            .read_line(&mut buffer)
            .expect("Reading from stdin failed");

        let value = buffer
            .trim()
            .parse::<i32>()
            .expect("Input was not an integer");
        self.registers.set(REG_V0, value as u32);
    }

    /// Read a string from stdin
    fn syscall_read_str(&mut self) {
        trace!("READ_STR");
        let output_address = self.registers.get(REG_A0);
        let max_length = self.registers.get(REG_A1);

        if max_length == 0 {
            return;
        }

        // Set up the byte stream
        let stdin = io::stdin();
        let stdin = stdin.lock();
        let mut found_newline = false;
        let bytes = stdin
            .bytes()
            // Reserve the last byte for the null byte
            .take(max_length as usize - 1)
            // Stop if we encounter a newline
            .take_while(|b| {
                found_newline = b.as_ref().map(|b| *b == b'\n').unwrap_or(false);
                !found_newline
            });

        // Write the bytes to memory
        let mut length = 0;
        for (i, byte) in bytes.enumerate() {
            let byte = byte.expect("Failed to read from stdin");
            self.memory.set(output_address + i as u32, byte);
            length += 1;
        }

        // Add the newline if we found one
        if found_newline {
            self.memory.set(output_address + length, b'\n');
            length += 1;
        }

        // Add the null byte
        self.memory.set(output_address + length, 0);
    }

    /// Exit with a code
    fn syscall_exit2(&mut self) {
        trace!("EXIT2");
        self.return_code = self.registers.get(REG_A0) as i32;
        self.running = false;
        trace!("Exit with code {}", self.return_code);
    }

    /// Print a value to stdout
    fn print(value: &[u8]) {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(value).expect("Writing to stdout failed");
        handle.flush().expect("Flushing stdout failed");
    }
}
