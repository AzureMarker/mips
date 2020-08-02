use crate::constants::{REG_A0, REG_V0, SYSCALL_PRINT_INT, SYSCALL_PRINT_STR, SYSCALL_READ_INT};
use crate::Processor;
use std::io;
use std::io::Write;

impl Processor {
    pub(crate) fn op_syscall(&mut self) {
        debug!("syscall");

        let operation = self.registers.get(REG_V0);

        match operation {
            SYSCALL_PRINT_INT => self.syscall_print_int(),
            SYSCALL_PRINT_STR => self.syscall_print_str(),
            SYSCALL_READ_INT => self.syscall_read_int(),
            _ => panic!("Unknown syscall operation {}", operation),
        }

        self.advance_program_counter();
    }

    fn syscall_print_int(&mut self) {
        let value = self.registers.get(REG_A0) as i32;
        Self::print(value.to_string().as_bytes());
    }

    fn syscall_print_str(&mut self) {
        let str_address = self.registers.get(REG_A0);
        let input_str = self.memory.get_str(str_address);

        Self::print(input_str.as_bytes());
    }

    fn syscall_read_int(&mut self) {
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

    fn print(value: &[u8]) {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(value).expect("Writing to stdout failed");
        handle.flush().expect("Flushing stdout failed");
    }
}
