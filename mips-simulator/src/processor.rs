use crate::constants::{FUNCTION_ADD, FUNCTION_BREAK, OP_ORI, OP_R_TYPE};
use crate::instruction::Instruction;
use crate::registers::Registers;

/// A MIPS processor
#[derive(Debug)]
pub struct Processor {
    registers: Registers,
}

impl Processor {
    pub fn new() -> Self {
        Processor {
            registers: Registers::new(),
        }
    }

    pub fn execute(&mut self, instruction: Instruction) {
        match instruction.op_code() {
            OP_R_TYPE => match instruction.function() {
                FUNCTION_ADD => self.add(instruction),
                FUNCTION_BREAK => self.break_fn(instruction),
                function => panic!("Unknown R-type function 0x{:02x}", function),
            },
            OP_ORI => self.ori(instruction),
            op_code => panic!("Unknown op code 0x{:02x}", op_code),
        }
    }

    fn add(&mut self, instruction: Instruction) {
        println!(
            "add {}, {}, {}",
            instruction.d_register(),
            instruction.s_register(),
            instruction.t_register()
        );
        let a = self.registers.get(instruction.s_register());
        let b = self.registers.get(instruction.t_register());
        self.registers.set(instruction.d_register(), a + b);
    }

    fn break_fn(&self, _instruction: Instruction) {
        println!("Executing a break instruction");
        // TODO: actually handle exceptions
    }

    fn ori(&mut self, instruction: Instruction) {
        println!(
            "ori {}, {}, {}",
            instruction.t_register(),
            instruction.s_register(),
            instruction.immediate()
        );
        let a = self.registers.get(instruction.s_register());
        let immediate = instruction.immediate() as u32;
        self.registers.set(instruction.t_register(), a | immediate);
    }
}
