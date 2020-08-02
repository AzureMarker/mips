use crate::constants::{REG_A0, REG_V0, SYSCALL_PRINT_STR, SYSCALL_READ_INT};
use crate::Processor;
use std::io;
use std::io::Write;

impl Processor {
    pub(crate) fn op_syscall(&mut self) {
        debug!("syscall");

        let operation = self.registers.get(REG_V0);

        match operation {
            SYSCALL_PRINT_STR => self.syscall_print_str(),
            SYSCALL_READ_INT => self.syscall_read_int(),
            _ => panic!("Unknown syscall operation {}", operation),
        }

        self.advance_program_counter();
    }

    fn syscall_print_str(&mut self) {
        let str_address = self.registers.get(REG_A0);
        let input_str = self.memory.get_str(str_address);

        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle
            .write_all(input_str.as_bytes())
            .expect("Writing to stdout failed");
        handle.flush().expect("Flushing stdout failed");
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
}
