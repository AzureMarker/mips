//! Lower the AST to IR

use crate::ast::{
    ConstantDef, Directive, Expr, ITypeOp, Instruction, Item, NumberDirective, Operation, Program,
    PseudoInstruction, RTypeOp, RepeatedExpr,
};
use crate::ir::{IrInstruction, IrProgram, Symbol, SymbolLocation};
use crate::string_unescape::unescape_str;
use mips_types::constants::{DATA_OFFSET, TEXT_OFFSET};
use std::collections::HashMap;
use std::fmt::{Display, LowerHex};
use std::iter;

type Constants = HashMap<String, i64>;
type SymbolTable = HashMap<String, Symbol>;

impl Program {
    pub fn lower(self) -> IrProgram {
        IrBuilder::default().build(self)
    }
}

/// Performs the two assembler passes
struct IrBuilder {
    instructions: Vec<IrInstruction>,
    data: Vec<u8>,
    symbol_table: SymbolTable,
    constants: Constants,
    globals: Vec<String>,
    current_section: SymbolLocation,
    text_offset: usize,
    text_words: HashMap<usize, u32>,
    auto_align: bool,
    current_label: Option<String>,
}

impl Default for IrBuilder {
    fn default() -> Self {
        Self {
            instructions: Vec::new(),
            data: Vec::new(),
            symbol_table: SymbolTable::new(),
            constants: Constants::new(),
            globals: Vec::new(),
            current_section: SymbolLocation::Text,
            text_offset: 0,
            text_words: HashMap::new(),
            auto_align: true,
            current_label: None,
        }
    }
}

impl IrBuilder {
    /// Build an IR program from the AST program
    fn build(mut self, program: Program) -> IrProgram {
        // First pass: find symbols and constants
        self.first_pass(&program);

        log::trace!("Constants: {:#?}", self.constants);
        log::trace!("Symbols: {:#?}", self.symbol_table);

        // Second pass: generate instruction IR
        self.second_pass(program);

        IrProgram {
            text: self.instructions,
            data: self.data,
            symbol_table: self.symbol_table,
            globals: self.globals,
        }
    }

    /// Run the first pass over the AST
    fn first_pass(&mut self, program: &Program) {
        for item in &program.items {
            let mut label_buffer = None;

            match item {
                Item::ConstantDef(constant) => self.visit_constant_def(constant),
                Item::Label(label) => {
                    label_buffer = Some(label.clone());
                    self.visit_label(label.clone());
                }
                Item::Directive(directive) => self.visit_directive(directive),
                Item::Instruction(instruction) => {
                    self.text_offset += 4 * instruction.expanded_size(&self.constants);
                }
            }

            self.current_label = label_buffer;
        }
    }

    /// Run the second pass over the AST
    fn second_pass(&mut self, program: Program) {
        for item in program.items {
            // Add in the text words we found in the first pass
            while let Some(word) = self.text_words.remove(&self.instructions.len()) {
                self.instructions.push(IrInstruction::Word(word));
            }

            if let Item::Instruction(instruction) = item {
                self.instructions.extend(instruction.lower(
                    self.instructions.len() * 4,
                    &self.constants,
                    &self.symbol_table,
                ));
            }
        }
    }

    fn visit_constant_def(&mut self, constant: &ConstantDef) {
        self.constants.insert(
            constant.name.clone(),
            constant
                .value
                .evaluate(&self.constants)
                .expect("Constants cannot have forward references"),
        );
    }

    fn visit_label(&mut self, label: String) {
        self.symbol_table.insert(
            label,
            Symbol {
                location: self.current_section,
                offset: match self.current_section {
                    SymbolLocation::Text => self.text_offset,
                    SymbolLocation::Data => self.data.len(),
                },
            },
        );
    }

    fn visit_directive(&mut self, directive: &Directive) {
        match directive {
            Directive::Text => {
                self.auto_align = true;
                self.current_section = SymbolLocation::Text;
            }
            Directive::Data => {
                self.auto_align = true;
                self.current_section = SymbolLocation::Data;
            }
            Directive::Global { label } => self.globals.push(label.clone()),
            Directive::Align { boundary } => self.visit_align(boundary),
            Directive::Space { size } => self.visit_space(size),
            Directive::NumberDirective { ty, values } => match ty {
                NumberDirective::Word => self.visit_word(values),
                NumberDirective::Byte => self.visit_byte(values),
            },
            Directive::Ascii { string, zero_pad } => self.visit_ascii(string, *zero_pad),
        }
    }

    fn visit_align(&mut self, boundary: &Expr) {
        let alignment = boundary
            .evaluate(&self.constants)
            .expect(".align cannot have forward references") as usize;

        if self.current_section == SymbolLocation::Text {
            if alignment <= 2 {
                // Only warn on alignments of 2 or less in the text segment.
                // This is not an error because only word-sized items are
                // allowed in the text segment so it's always aligned to <= 2.
                log::warn!(".align does nothing in the text segment");
                return;
            } else {
                // An alignment greater than 2 does not make sense in the text
                // segment.
                panic!("Cannot use .align of 2 or greater in the text segment");
            }
        }

        if alignment == 0 {
            self.auto_align = false;
        } else {
            self.align_section(alignment);
        }
    }

    fn visit_space(&mut self, size: &Expr) {
        let section_data = match self.current_section {
            SymbolLocation::Text => panic!("Cannot use .space in the text segment"),
            SymbolLocation::Data => &mut self.data,
        };

        // FIXME: check if value is negative
        let size = size
            .evaluate(&self.constants)
            .expect(".space cannot have forward references") as usize;

        section_data.extend(iter::repeat(0).take(size));
    }

    fn visit_byte(&mut self, values: &[RepeatedExpr]) {
        self.visit_number(
            values,
            RepeatedExpr::as_bytes,
            |_, _| panic!("Cannot use .byte in the text segment"),
            Some,
        )
    }

    fn visit_word(&mut self, values: &[RepeatedExpr]) {
        self.visit_number(
            values,
            RepeatedExpr::as_words,
            |builder, words| {
                for word in words {
                    builder.text_words.insert(builder.text_offset, word);
                    builder.text_offset += 4;
                }
            },
            |word| word.to_be_bytes().to_vec(),
        )
    }

    /// A general version of a directive which inserts numbers into a data segment,
    /// such as .word or .float.
    fn visit_number<T, I1: IntoIterator<Item = T>, I2: IntoIterator<Item = u8>>(
        &mut self,
        values: &[RepeatedExpr],
        as_iterator: impl Fn(&RepeatedExpr, &Constants) -> I1,
        handle_text: impl FnOnce(&mut Self, Vec<T>),
        to_bytes: impl Fn(T) -> I2,
    ) {
        if self.auto_align && self.current_section != SymbolLocation::Text {
            self.align_section(2);
        }

        let numbers: Vec<T> = values
            .iter()
            .flat_map(|e| as_iterator(e, &self.constants))
            .collect();

        match self.current_section {
            SymbolLocation::Text => {
                handle_text(self, numbers);
            }
            _ => {
                let section_data = match self.current_section {
                    SymbolLocation::Text => unreachable!(),
                    SymbolLocation::Data => &mut self.data,
                };

                section_data.extend(numbers.into_iter().flat_map(to_bytes));
            }
        }
    }

    fn visit_ascii(&mut self, string: &str, zero_pad: bool) {
        let section_data = match self.current_section {
            SymbolLocation::Text => {
                panic!(
                    "Cannot use .ascii{} in the text segment",
                    if zero_pad { "z" } else { "" }
                )
            }
            SymbolLocation::Data => &mut self.data,
        };

        if !string.is_ascii() {
            // TODO: return a proper error
            panic!("Strings must be ASCII");
        }

        // TODO: handle the error
        let unescaped = unescape_str(string).unwrap();

        if zero_pad {
            section_data.extend(unescaped.bytes().chain(iter::once(0)));
        } else {
            section_data.extend(unescaped.bytes().chain(iter::empty()));
        }
    }

    /// Aligns the current section according to the alignment value. If there
    /// was a label pointing at this directive, it is realigned.
    fn align_section(&mut self, alignment: usize) {
        let section = match self.current_section {
            SymbolLocation::Text => panic!("Cannot align the text segment"),
            SymbolLocation::Data => &mut self.data,
        };
        let step_size = usize::pow(2, alignment as u32);

        if section.len() % step_size != 0 {
            let alignment_amount = step_size - (section.len() % step_size);
            section.extend(iter::repeat(0).take(alignment_amount));

            // If there was a label pointing at this directive, realign it
            if let Some(label) = &self.current_label {
                self.symbol_table.get_mut(label).unwrap().offset += alignment_amount;
            }
        }
    }
}

impl Expr {
    pub fn evaluate(&self, constants: &Constants) -> Result<i64, String> {
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

impl RepeatedExpr {
    fn as_bytes(&self, constants: &Constants) -> impl Iterator<Item = u8> {
        self.as_iterator(constants, ".byte", 2, |value| {
            let truncated = value as i8;
            (truncated as u8, truncated as i64 == value)
        })
    }

    fn as_words(&self, constants: &Constants) -> impl Iterator<Item = u32> {
        self.as_iterator(constants, ".word", 8, |value| {
            let truncated = value as i32;
            (truncated as u32, truncated as i64 == value)
        })
    }

    /// Convert this repeated expression into a stream of truncated numbers
    fn as_iterator<T: Display + LowerHex + Copy>(
        &self,
        constants: &Constants,
        directive: &'static str,
        format_width: usize,
        truncate: impl FnOnce(i64) -> (T, bool),
    ) -> impl Iterator<Item = T> {
        let value = self
            .expr
            .evaluate(&constants)
            .expect(".word cannot have forward references");
        let times = self
            .times
            .evaluate(&constants)
            .expect(".word cannot have forward references") as usize; // FIXME: check for negative repeat value

        // Values are explicitly truncated.
        let (truncated, is_same) = truncate(value);
        if !is_same {
            // TODO: give more info, like a line number
            log::warn!(
                "{}: Truncated 0x{:016x} to 0x{:0width$x}",
                directive,
                value,
                truncated,
                width = format_width
            );
        }

        iter::repeat(truncated).take(times)
    }
}

impl Instruction {
    /// Get the number of instructions this instruction expands to
    pub fn expanded_size(&self, constants: &Constants) -> usize {
        match self {
            Instruction::RType { .. } | Instruction::IType { .. } | Instruction::JType { .. } => 1,
            Instruction::Pseudo(pseduo) => pseduo.expanded_size(constants),
        }
    }

    pub fn lower(
        self,
        current_offset: usize,
        constants: &Constants,
        symbol_table: &SymbolTable,
    ) -> Vec<IrInstruction> {
        match self {
            Instruction::RType {
                op_code,
                rs,
                rt,
                rd,
                shift,
            } => vec![IrInstruction::RType {
                op_code,
                rs: rs.index().unwrap(),
                rt: rt.index().unwrap(),
                rd: rd.index().unwrap(),
                // TODO: check if shift is too big
                shift: shift.evaluate(constants).unwrap() as u8,
            }],
            Instruction::IType {
                op_code,
                rs,
                rt,
                immediate,
            } if op_code.needs_offset() => {
                // FIXME: this only supports literal labels or integer offsets
                let offset = match immediate {
                    Expr::Constant(label) => constants
                        .get(&label)
                        .map(|&value| value as i16)
                        .or_else(|| symbol_table.get(&label).map(|symbol| {
                            assert_eq!(symbol.location, SymbolLocation::Text, "Can only branch to labels in the text section");
                            // FIXME: make sure the offset isn't too big
                            // Divide by four because it's counted in instructions to skip,
                            // minus one because the offset affects the next PC
                            ((symbol.offset as isize - current_offset as isize) / 4 - 1) as i16
                        }))
                        .unwrap_or_else(|| panic!("Unable to find '{}'", label)),
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
            Instruction::IType {
                op_code,
                rs,
                rt,
                immediate,
            } => {
                // Non-offset based I-type instructions
                vec![IrInstruction::IType {
                    op_code,
                    rs: rs.index().unwrap(),
                    rt: rt.index().unwrap(),
                    // FIXME: make sure the constant is not too big
                    immediate: immediate.evaluate(constants).unwrap() as i16,
                }]
            }
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
            Instruction::Pseudo(pseudo_instruction) => {
                pseudo_instruction.lower(constants, symbol_table)
            }
        }
    }
}

impl PseudoInstruction {
    /// Get the number of instructions this pseudo-instruction expands to
    pub fn expanded_size(&self, constants: &Constants) -> usize {
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

    pub fn lower(self, constants: &Constants, symbol_table: &SymbolTable) -> Vec<IrInstruction> {
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
