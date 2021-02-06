//! Lower the AST to IR

use crate::ast::{
    ConstantDef, Directive, Expr, ITypeOp, Instruction, Item, Operation, Program,
    PseudoInstruction, RTypeOp,
};
use crate::ir::{IrData, IrInstruction, IrProgram, IrText, Symbol, SymbolLocation, SymbolTable};
use std::collections::HashMap;

impl Program {
    pub fn lower(self) -> IrProgram {
        let mut instructions: Vec<IrInstruction> = Vec::new();
        let mut data: Vec<u8> = Vec::new();
        let mut symbol_table: HashMap<String, Symbol> = HashMap::new();
        let mut globals: Vec<String> = Vec::new();
        let mut constants: HashMap<String, i64> = HashMap::new();

        let mut current_section = SymbolLocation::Text;
        let mut text_offset = 0;

        // Find symbols and constants
        for item in &self.items {
            match item {
                Item::ConstantDef(ConstantDef { name, value }) => {
                    // Constants cannot have forward references
                    constants.insert(name.clone(), value.evaluate(&constants));
                }
                Item::Label(name) => {
                    symbol_table.insert(
                        name.clone(),
                        Symbol {
                            location: current_section,
                            offset: match current_section {
                                SymbolLocation::Text => text_offset,
                                SymbolLocation::Data => data.len(),
                            },
                        },
                    );
                }
                Item::Directive(directive) => match directive {
                    // FIXME: We're not checking what section we're in for directives like
                    //        align or space.
                    Directive::Text => current_section = SymbolLocation::Text,
                    Directive::Data => current_section = SymbolLocation::Data,
                    Directive::Global { label } => globals.push(label.clone()),
                    Directive::Align { boundary } => {
                        // Boundaries cannot have forward references
                        let boundary = boundary.evaluate(&constants) as usize;

                        if boundary == 0 || data.len() % boundary == 0 {
                            // FIXME: I don't think we're properly handling boundaries
                            continue;
                        }

                        data.extend(std::iter::repeat(0).take(boundary - (data.len() % boundary)));
                    }
                    Directive::Space { size } => {
                        // Space cannot have forward references
                        data.extend(std::iter::repeat(0).take(size.evaluate(&constants) as usize));
                    }
                    Directive::Word { values } => data.extend(
                        values
                            .iter()
                            // Words cannot have forward references
                            .flat_map(|e| e.evaluate(&constants).to_be_bytes().to_vec()),
                    ),
                    Directive::Asciiz { string } => {
                        // TODO: enforce only ASCII?
                        data.extend(string.bytes().chain(std::iter::once(0)))
                    }
                },
                Item::Instruction(instruction) => {
                    text_offset += 4 * instruction.expanded_size();
                }
            }
        }

        // Second pass: generate instruction IR
        for item in self.items {
            match item {
                Item::ConstantDef(_) => {}
                Item::Label(_) => {}
                Item::Directive(directive) => match directive {
                    // FIXME: atm we don't do anything with directives on second
                    //        pass, but should we?
                    Directive::Text
                    | Directive::Data
                    | Directive::Global { .. }
                    | Directive::Align { .. }
                    | Directive::Space { .. }
                    | Directive::Word { .. }
                    | Directive::Asciiz { .. } => {}
                },
                Item::Instruction(instruction) => {
                    instructions.extend(instruction.lower(&constants))
                }
            }
        }

        IrProgram {
            text: IrText { instructions },
            data: IrData { data },
            symbol_table: SymbolTable { map: symbol_table },
            globals,
        }
    }
}

impl Expr {
    pub fn evaluate(&self, constants: &HashMap<String, i64>) -> i64 {
        match self {
            Expr::Number(num) => *num,
            Expr::Constant(name) => *constants
                .get(name)
                // TODO: return a proper error
                .unwrap_or_else(|| panic!("Unable to find constant '{}'", name)),
            Expr::Calculated {
                operation,
                left,
                right,
            } => {
                let left = left.evaluate(constants);
                let right = right.evaluate(constants);
                match operation {
                    Operation::Add => left + right,
                    Operation::Subtract => left - right,
                    Operation::Multiply => left * right,
                    Operation::Divide => left / right,
                }
            }
            Expr::Negated(expr) => -expr.evaluate(constants),
        }
    }
}

impl Instruction {
    /// Get the number of instructions this instruction expands to
    pub fn expanded_size(&self) -> usize {
        match self {
            Instruction::RType { .. }
            | Instruction::IType { .. }
            | Instruction::JType { .. }
            | Instruction::Syscall => 1,
            Instruction::Pseudo(pseduo) => pseduo.expanded_size(),
        }
    }

    pub fn lower(self, constants: &HashMap<String, i64>) -> Vec<IrInstruction> {
        match self {
            Instruction::RType {
                op_code,
                rs,
                rt,
                rd,
            } => vec![IrInstruction::RType {
                op_code,
                rs: rs.index().unwrap(),
                rt: rt.index().unwrap(),
                rd: rd.index().unwrap(),
                shift: 0, // FIXME: add to AST, needed by some unimplemented ops
            }],
            Instruction::IType {
                op_code,
                rs,
                rt,
                immediate,
            } => vec![IrInstruction::IType {
                op_code,
                rs: rs.index().unwrap(),
                rt: rt.index().unwrap(),
                // FIXME: make sure the constant is not too big
                immediate: immediate.evaluate(&constants) as i16,
            }],
            Instruction::JType { op_code, label } => vec![IrInstruction::JType {
                op_code,
                pseudo_address: 0xDEADBEEF, // TODO: fix this
            }],
            Instruction::Syscall => vec![IrInstruction::Syscall],
            Instruction::Pseudo(pseudo_instruction) => pseudo_instruction.lower(constants),
        }
    }
}

impl PseudoInstruction {
    /// Get the number of instructions this pseudo-instruction expands to
    pub fn expanded_size(&self) -> usize {
        match self {
            PseudoInstruction::LoadImmediate { .. } | PseudoInstruction::LoadAddress { .. } => 2,
            PseudoInstruction::Move { .. } => 1,
        }
    }

    pub fn lower(self, constants: &HashMap<String, i64>) -> Vec<IrInstruction> {
        match self {
            PseudoInstruction::LoadImmediate { rd, value } => {
                let value = value.evaluate(constants) as u32;

                // FIXME: this assumes it's a 32 bit immediate, but we could
                //        optimize to one instruction if it's 16 bit. We also
                //        need to check that it's not bigger than 32 bits.
                Self::load_u32_into_register(rd.index().unwrap(), value)
            }
            PseudoInstruction::LoadAddress { rd, label: _label } => {
                // FIXME: note that this instruction references a label so the
                //        address can be updated once we know the label's address.
                Self::load_u32_into_register(rd.index().unwrap(), 0xDEADBEEF)
            }
            PseudoInstruction::Move { rs, rt } => vec![IrInstruction::RType {
                op_code: RTypeOp::Or,
                rs: rs.index().unwrap(),
                rt: rt.index().unwrap(),
                rd: 0,
                shift: 0,
            }],
        }
    }

    fn load_u32_into_register(register: u8, value: u32) -> Vec<IrInstruction> {
        vec![
            IrInstruction::IType {
                op_code: ITypeOp::Lui,
                rs: 0,
                rt: register,
                immediate: (value >> 16) as i16,
            },
            IrInstruction::IType {
                op_code: ITypeOp::Ori,
                rs: 0,
                rt: register,
                immediate: value as i16,
            },
        ]
    }
}
