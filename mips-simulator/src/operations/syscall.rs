use crate::constants::{REG_A0, REG_V0, SYSCALL_PRINT_STR};
use crate::Processor;

impl Processor {
    pub(crate) fn op_syscall(&mut self) {
        debug!("syscall");

        let operation = self.registers.get(REG_V0);

        match operation {
            SYSCALL_PRINT_STR => self.syscall_print_str(),
            _ => panic!("Unknown syscall operation {}", operation),
        }

        self.advance_program_counter();
    }

    fn syscall_print_str(&mut self) {
        let str_address = self.registers.get(REG_A0);
        let input_str = self.memory.get_str(str_address);
        let input_str = input_str
            .to_str()
            .expect("PRINT_STR syscall input is not valid UTF-8");
        print!("{}", input_str);
    }
}
