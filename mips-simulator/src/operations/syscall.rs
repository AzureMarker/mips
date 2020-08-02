use crate::constants::{
    REG_A0, REG_V0, SYSCALL_EXIT2, SYSCALL_PRINT_INT, SYSCALL_PRINT_STR, SYSCALL_READ_INT,
};
use crate::Processor;
use std::io;
use std::io::Write;

impl Processor {
    /// Handle a syscall operation
    pub(crate) fn op_syscall(&mut self) {
        debug!("syscall");

        match self.registers.get(REG_V0) {
            SYSCALL_PRINT_INT => self.syscall_print_int(),
            SYSCALL_PRINT_STR => self.syscall_print_str(),
            SYSCALL_READ_INT => self.syscall_read_int(),
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
