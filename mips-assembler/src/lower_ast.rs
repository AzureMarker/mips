//! Lower the AST to IR

use crate::ast::{
    ConstantDef, Directive, Expr, ITypeOp, Instruction, Item, Operation, Program,
    PseudoInstruction, RTypeOp,
};
use crate::ir::{IrInstruction, IrProgram, Symbol, SymbolLocation};
use crate::string_unescape::unescape_str;
use mips_types::constants::{DATA_OFFSET, TEXT_OFFSET};
use std::collections::HashMap;
use std::iter;

impl Program {
    pub fn lower(self) -> IrProgram {
        let mut instructions: Vec<IrInstruction> = Vec::new();
        let mut data: Vec<u8> = Vec::new();
        let mut symbol_table: HashMap<String, Symbol> = HashMap::new();
        let mut globals: Vec<String> = Vec::new();
        let mut constants: HashMap<String, i64> = HashMap::new();

        let mut current_section = SymbolLocation::Text;
        let mut text_offset = 0;
        let mut alignment = 0;

        // Find symbols and constants
        for item in &self.items {
            match item {
                Item::ConstantDef(ConstantDef { name, value }) => {
                    constants.insert(
                        name.clone(),
                        value
                            .evaluate(&constants)
                            .expect("Constants cannot have forward references"),
                    );
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
                    // FIXME: We're not checking what section we're in for
                    //        directives that allocate memory
                    Directive::Text => current_section = SymbolLocation::Text,
                    Directive::Data => current_section = SymbolLocation::Data,
                    Directive::Global { label } => globals.push(label.clone()),
                    Directive::Align { boundary } => {
                        alignment = boundary
                            .evaluate(&constants)
                            .expect(".align cannot have forward references")
                            as usize;
                    }
                    Directive::Space { size } => {
                        align_section(&mut data, alignment);

                        // FIXME: check if value is negative
                        let size = size
                            .evaluate(&constants)
                            .expect(".space cannot have forward references")
                            as usize;

                        data.extend(iter::repeat(0).take(size));
                    }
                    Directive::Word { values } => {
                        align_section(&mut data, alignment);

                        data.extend(values.iter().flat_map(|e| {
                            // FIXME: check if value is too big
                            let value = e
                                .evaluate(&constants)
                                .expect(".word cannot have forward references")
                                as u32;

                            value.to_be_bytes().to_vec()
                        }))
                    }
                    Directive::Asciiz { string } => {
                        align_section(&mut data, alignment);

                        // TODO: handle the error
                        let unescaped = unescape_str(string).unwrap();

                        // TODO: enforce only ASCII?
                        data.extend(unescaped.bytes().chain(std::iter::once(0)))
                    }
                },
                Item::Instruction(instruction) => {
                    text_offset += 4 * instruction.expanded_size(&constants);
                }
            }
        }

        println!("Constants: {:#?}\nSymbols: {:#?}", constants, symbol_table);

        // Second pass: generate instruction IR
        for item in self.items {
            match item {
                Item::ConstantDef(_) | Item::Label(_)
                // FIXME: atm we don't do anything with directives on second
                //        pass, but should we?
                | Item::Directive(_) => {}
                Item::Instruction(instruction) => instructions.extend(instruction.lower(
                    instructions.len() * 4,
                    &constants,
                    &symbol_table,
                )),
            }
        }

        IrProgram {
            text: instructions,
            data,
            symbol_table,
            globals,
        }
    }
}

fn align_section(section: &mut Vec<u8>, alignment: usize) {
    let step_size = usize::pow(2, alignment as u32);

    if section.len() % step_size != 0 {
        section.extend(iter::repeat(0).take(step_size - (section.len() % step_size)));
    }
}

impl Expr {
    pub fn evaluate(&self, constants: &HashMap<String, i64>) -> Result<i64, String> {
        match self {
            Expr::Number(num) => Ok(*num),
            Expr::Constant(name) => constants
                .get(name)
                .copied()
                // TODO: return a proper error
                .ok_or_else(|| format!("Unable to find constant '{}'", name)),
            Expr::Calculated {
                operation,
                left,
                right,
            } => {
                let left = left.evaluate(constants)?;
                let right = right.evaluate(constants)?;
                Ok(match operation {
                    Operation::Add => left + right,
                    Operation::Subtract => left - right,
                    Operation::Multiply => left * right,
                    Operation::Divide => left / right,
                    Operation::BitwiseShiftLeft => ((left as u64) << right as u64) as i64,
                    Operation::BitwiseShiftRight => ((left as u64) >> right as u64) as i64,
                    Operation::BitwiseAnd => left & right,
                    Operation::BitwiseXor => left ^ right,
                    Operation::BitwiseOr => left | right,
                })
            }
            Expr::Negated(expr) => Ok(-expr.evaluate(constants)?),
        }
    }
}

impl Instruction {
    /// Get the number of instructions this instruction expands to
    pub fn expanded_size(&self, constants: &HashMap<String, i64>) -> usize {
        match self {
            Instruction::RType { .. }
            | Instruction::IType { .. }
            | Instruction::JType { .. }
            | Instruction::Syscall => 1,
            Instruction::Pseudo(pseduo) => pseduo.expanded_size(constants),
        }
    }

    pub fn lower(
        self,
        current_offset: usize,
        constants: &HashMap<String, i64>,
        symbol_table: &HashMap<String, Symbol>,
    ) -> Vec<IrInstruction> {
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
            } => match op_code {
                // Branching instructions need offsets
                ITypeOp::Beq => {
                    // FIXME: this only supports literal labels or integer offsets
                    let offset = match immediate {
                        Expr::Constant(label) => {
                            constants.get(&label).map(|&value| value as i16)
                                .or_else(|| symbol_table.get(&label).map(|symbol| {
                                    assert_eq!(symbol.location, SymbolLocation::Text, "Can only branch to labels in the text section");
                                    // FIXME: make sure the offset isn't too big
                                    // Divide by four because it's counted in instructions to skip,
                                    // minus one because the offset affects the next PC
                                    ((symbol.offset as isize - current_offset as isize) / 4 - 1) as i16
                                }))
                                .unwrap_or_else(|| panic!("Unable to find '{}'", label))
                        },
                        Expr::Number(offset) => offset as i16,
                        _ => panic!("Only labels, constants, and numbers are currently allowed in branching instructions")
                    };

                    vec![IrInstruction::IType {
                        op_code,
                        rs: rs.index().unwrap(),
                        rt: rt.index().unwrap(),
                        immediate: offset,
                    }]
                }
                // Other I-types use the written-down value
                ITypeOp::Addi
                | ITypeOp::Lui
                | ITypeOp::Lw
                | ITypeOp::Ori
                | ITypeOp::Slti
                | ITypeOp::Sw => vec![IrInstruction::IType {
                    op_code,
                    rs: rs.index().unwrap(),
                    rt: rt.index().unwrap(),
                    // FIXME: make sure the constant is not too big
                    immediate: immediate.evaluate(constants).unwrap() as i16,
                }],
            },
            Instruction::JType { op_code, label } => {
                let pseudo_address = match label {
                    Expr::Constant(label) => {
                        let symbol = symbol_table
                            .get(&label)
                            .unwrap_or_else(|| panic!("Could not find symbol '{}'", label));

                        symbol.pseudo_address()
                    }
                    // FIXME: make sure the constant is not too large or negative
                    Expr::Number(address) => address as u32,
                    _ => panic!("Only labels and raw addresses are currently allowed in J-type instructions")
                };

                vec![IrInstruction::JType {
                    op_code,
                    pseudo_address,
                }]
            }
            Instruction::Syscall => vec![IrInstruction::Syscall],
            Instruction::Pseudo(pseudo_instruction) => {
                pseudo_instruction.lower(constants, symbol_table)
            }
        }
    }
}

impl PseudoInstruction {
    /// Get the number of instructions this pseudo-instruction expands to
    pub fn expanded_size(&self, constants: &HashMap<String, i64>) -> usize {
        match self {
            PseudoInstruction::LoadImmediate { value, .. } => {
                let value = value
                    .evaluate(constants)
                    .expect("li cannot have forward references") as u32;

                // We can fit an li into one instruction if the upper or lower
                // 16 bits are zero.
                if value >> 16 & 0xFFFF == 0 || value & 0xFFFF == 0 {
                    1
                } else {
                    2
                }
            }
            PseudoInstruction::LoadAddress { .. } => 2,
            PseudoInstruction::Move { .. } => 1,
        }
    }

    pub fn lower(
        self,
        constants: &HashMap<String, i64>,
        symbol_table: &HashMap<String, Symbol>,
    ) -> Vec<IrInstruction> {
        match self {
            PseudoInstruction::LoadImmediate { rd, value } => {
                let value = value.evaluate(constants).unwrap() as u32;

                if value >> 16 & 0xFFFF == 0 {
                    // If the upper 16 bits are zero, we only need to load the
                    // lower bits
                    vec![IrInstruction::IType {
                        op_code: ITypeOp::Ori,
                        rs: 0,
                        rt: rd.index().unwrap(),
                        immediate: value as i16,
                    }]
                } else if value & 0xFFFF == 0 {
                    // If the lower 16 bits are zero, we only need to load the
                    // upper bits
                    vec![IrInstruction::IType {
                        op_code: ITypeOp::Lui,
                        rs: 0,
                        rt: rd.index().unwrap(),
                        immediate: (value >> 16) as i16,
                    }]
                } else {
                    Self::load_u32_into_register(rd.index().unwrap(), value)
                }
            }
            PseudoInstruction::LoadAddress { rd, label } => {
                let symbol = symbol_table
                    .get(&label)
                    .unwrap_or_else(|| panic!("Could not find symbol '{}'", label));

                Self::load_u32_into_register(rd.index().unwrap(), symbol.address())
            }
            PseudoInstruction::Move { rs, rt } => vec![IrInstruction::RType {
                op_code: RTypeOp::Or,
                rs: rs.index().unwrap(),
                rt: 0,
                rd: rt.index().unwrap(),
                shift: 0,
            }],
        }
    }

    /// This loads the upper half into the $at register and then ORs it with the
    /// lower half and outputs to the destination register.
    fn load_u32_into_register(register: u8, value: u32) -> Vec<IrInstruction> {
        vec![
            IrInstruction::IType {
                op_code: ITypeOp::Lui,
                rs: 0,
                rt: 1,
                immediate: (value >> 16) as i16,
            },
            IrInstruction::IType {
                op_code: ITypeOp::Ori,
                rs: 1,
                rt: register,
                immediate: value as i16,
            },
        ]
    }
}

impl Symbol {
    /// Calculate the address of the symbol
    fn address(&self) -> u32 {
        match self.location {
            SymbolLocation::Text => TEXT_OFFSET + self.offset as u32,
            SymbolLocation::Data => DATA_OFFSET + self.offset as u32,
        }
    }

    /// Calculate the pseudo-address of the symbol
    /// (upper four bits and lower two bits removed)
    fn pseudo_address(&self) -> u32 {
        (self.address() & 0x0FFFFFFC) >> 2
    }
}
