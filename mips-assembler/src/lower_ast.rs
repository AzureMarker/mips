//! Lower the AST to IR

use crate::ast::{
    ConstantDef, Directive, Expr, ITypeOp, Instruction, Item, NumberDirective, Operation, Program,
    PseudoInstruction, RTypeOp, Register, RepeatedExpr,
};
use crate::ir::{
    IrInstruction, IrProgram, ReferenceEntry, ReferenceMethod, ReferenceTarget, ReferenceType,
    RelocationEntry, RelocationType, Symbol, SymbolLocation, SymbolType,
};
use crate::string_table::StringTable;
use crate::string_unescape::unescape_str;
use either::Either;
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
    rdata: Vec<u8>,
    sdata: Vec<u8>,
    symbol_table: SymbolTable,
    string_table: StringTable,
    relocation: Vec<RelocationEntry>,
    references: Vec<ReferenceEntry>,
    constants: Constants,
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
            rdata: Vec::new(),
            sdata: Vec::new(),
            symbol_table: SymbolTable::new(),
            string_table: StringTable::new(),
            relocation: Vec::new(),
            references: Vec::new(),
            constants: Constants::new(),
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

        // Second pass: generate instruction IR
        self.second_pass(program);

        IrProgram {
            text: self.instructions,
            data: self.data,
            rdata: self.rdata,
            sdata: self.sdata,
            symbol_table: self.symbol_table,
            relocation: self.relocation,
            references: self.references,
            string_table: self.string_table,
        }
    }

    /// Run the first pass over the AST
    fn first_pass(&mut self, program: &Program) {
        for item in &program.items {
            let mut label_buffer = None;

            match item {
                Item::ConstantDef(constant) => {
                    // In the case where a constant definition is between a
                    // label and a auto-aligning directive, make sure we
                    // remember the label after visiting the constant defs. Ex:
                    //
                    //     .half 1
                    // my_label: # This would be aligned on a 2 byte boundary
                    // MY_CONSTANT = 2
                    //     .word 3 # Auto-aligns to a 4 byte boundary, moving the label
                    label_buffer = self.current_label.clone();
                    self.visit_constant_def(constant)
                }
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
                    &mut self.relocation,
                    &mut self.references,
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
        let offset = self.current_offset();

        if let Some(symbol) = self.symbol_table.get_mut(&label) {
            match symbol.ty {
                SymbolType::Local | SymbolType::Export => {
                    // TODO: return a proper error
                    panic!("Found duplicate definition of {}", label);
                }
                SymbolType::Import => {
                    // We first found the symbol in .globl and assumed it was an import
                    // because we hadn't seen it yet. Now that we found its declaration,
                    // we know that it is an export.
                    symbol.ty = SymbolType::Export;
                    symbol.location = self.current_section;
                    symbol.offset = offset;
                }
            }
        } else {
            // This is a never-before-seen label
            self.symbol_table.insert(
                label.clone(),
                Symbol {
                    location: self.current_section,
                    offset,
                    string_offset: self.string_table.insert(label),
                    ty: SymbolType::Local,
                },
            );
        }
    }

    fn current_offset(&self) -> usize {
        match self.current_section {
            SymbolLocation::Text => self.text_offset,
            SymbolLocation::Data => self.data.len(),
            SymbolLocation::RData => self.rdata.len(),
            SymbolLocation::SData => self.sdata.len(),
        }
    }

    fn visit_directive(&mut self, directive: &Directive) {
        match directive {
            Directive::Text => self.set_section(SymbolLocation::Text),
            Directive::Data => self.set_section(SymbolLocation::Data),
            Directive::RData => self.set_section(SymbolLocation::RData),
            Directive::SData => self.set_section(SymbolLocation::SData),
            Directive::Global { label } => {
                if let Some(symbol) = self.symbol_table.get_mut(label) {
                    match symbol.ty {
                        SymbolType::Local => symbol.ty = SymbolType::Export,
                        SymbolType::Import | SymbolType::Export => {
                            // TODO: return a proper error
                            panic!("Found duplicate .globl {}", label);
                        }
                    }
                } else {
                    // Never-before-seen label, assume it is an import for now
                    self.symbol_table.insert(
                        label.clone(),
                        Symbol {
                            location: self.current_section,
                            offset: self.current_offset(),
                            string_offset: self.string_table.insert(label.clone()),
                            ty: SymbolType::Import,
                        },
                    );
                }
            }
            Directive::Align { boundary } => self.visit_align(boundary),
            Directive::Space { size } => self.visit_space(size),
            Directive::NumberDirective { ty, values } => match ty {
                NumberDirective::Word => self.visit_word(values),
                NumberDirective::Half => self.visit_half(values),
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
            SymbolLocation::RData => &mut self.rdata,
            SymbolLocation::SData => &mut self.sdata,
        };

        // FIXME: check if value is negative
        let size = size
            .evaluate(&self.constants)
            .expect(".space cannot have forward references") as usize;

        section_data.extend(iter::repeat(0).take(size));
    }

    fn visit_byte(&mut self, values: &[RepeatedExpr]) {
        let numbers = values
            .iter()
            .flat_map(|e| e.as_bytes(&self.constants))
            .collect();

        self.extend_with_numbers(
            numbers,
            |_, _| panic!("Cannot use .byte in the text segment"),
            Some,
        )
    }

    fn visit_half(&mut self, values: &[RepeatedExpr]) {
        self.auto_align(1);

        let numbers = values
            .iter()
            .flat_map(|e| e.as_halves(&self.constants))
            .collect();

        self.extend_with_numbers(
            numbers,
            |_, _| panic!("Cannot use .half in the text segment"),
            |half| half.to_be_bytes().to_vec(),
        )
    }

    fn visit_word(&mut self, values: &[RepeatedExpr]) {
        self.auto_align(2);

        let numbers = values
            .iter()
            .flat_map(|e| {
                // .word can reference constants or labels
                let value = e
                    .expr
                    .evaluate(&self.constants)
                    .ok()
                    .or_else(|| match &e.expr {
                        Expr::Constant(label) => {
                            let symbol = self.symbol_table.get(label)?;

                            match symbol.ty {
                                SymbolType::Local | SymbolType::Export => {
                                    self.relocation.push(RelocationEntry {
                                        offset: self.current_offset(),
                                        location: self.current_section,
                                        relocation_type: RelocationType::Word,
                                    });
                                }
                                SymbolType::Import => {
                                    self.references.push(ReferenceEntry {
                                        offset: self.current_offset(),
                                        location: self.current_section,
                                        str_idx: symbol.string_offset,
                                        reference_type: ReferenceType {
                                            target: ReferenceTarget::Word,
                                            method: ReferenceMethod::Replace,
                                        },
                                    });
                                }
                            }

                            Some(symbol.offset as i64)
                        }
                        _ => None,
                    })
                    .expect(".word cannot have forward references");

                e.as_words(value, &self.constants)
            })
            .collect();

        self.extend_with_numbers(
            numbers,
            |builder, words| {
                for word in words {
                    builder.text_words.insert(builder.text_offset, word);
                    builder.text_offset += 4;
                }
            },
            |word| word.to_be_bytes().to_vec(),
        );
    }

    fn auto_align(&mut self, alignment: usize) {
        if self.auto_align && self.current_section != SymbolLocation::Text {
            self.align_section(alignment);
        }
    }

    fn extend_with_numbers<T, I: IntoIterator<Item = u8>>(
        &mut self,
        numbers: Vec<T>,
        handle_text: impl FnOnce(&mut Self, Vec<T>),
        to_bytes: impl Fn(T) -> I,
    ) {
        let section_data = match self.current_section {
            SymbolLocation::Text => {
                handle_text(self, numbers);
                return;
            }
            SymbolLocation::Data => &mut self.data,
            SymbolLocation::RData => &mut self.rdata,
            SymbolLocation::SData => &mut self.sdata,
        };

        section_data.extend(numbers.into_iter().flat_map(to_bytes));
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
            SymbolLocation::RData => &mut self.rdata,
            SymbolLocation::SData => &mut self.sdata,
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

    fn set_section(&mut self, location: SymbolLocation) {
        self.auto_align = true;
        self.current_section = location;
    }

    /// Aligns the current section according to the alignment value. If there
    /// was a label pointing at this directive, it is realigned.
    fn align_section(&mut self, alignment: usize) {
        let section = match self.current_section {
            SymbolLocation::Text => panic!("Cannot align the text segment"),
            SymbolLocation::Data => &mut self.data,
            SymbolLocation::RData => &mut self.rdata,
            SymbolLocation::SData => &mut self.sdata,
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
            Expr::BitwiseNegated(expr) => Ok(expr.evaluate(constants)? ^ -1),
        }
    }
}

impl RepeatedExpr {
    fn as_bytes(&self, constants: &Constants) -> impl Iterator<Item = u8> {
        self.as_iterator(
            self.get_const_value(constants, ".bytes"),
            constants,
            ".byte",
            2,
            |value| {
                let truncated = value as i8;
                (truncated as u8, truncated as i64 == value)
            },
        )
    }

    fn as_halves(&self, constants: &Constants) -> impl Iterator<Item = u16> {
        self.as_iterator(
            self.get_const_value(constants, ".half"),
            constants,
            ".half",
            4,
            |value| {
                let truncated = value as i16;
                (truncated as u16, truncated as i64 == value)
            },
        )
    }

    fn as_words(&self, value: i64, constants: &Constants) -> impl Iterator<Item = u32> {
        self.as_iterator(value, constants, ".word", 8, |value| {
            let truncated = value as i32;
            (truncated as u32, truncated as i64 == value)
        })
    }

    fn get_const_value(&self, constants: &Constants, directive: &'static str) -> i64 {
        self.expr
            .evaluate(&constants)
            .unwrap_or_else(|_| panic!("{} cannot have forward references", directive))
    }

    /// Convert this repeated expression into a stream of truncated numbers
    fn as_iterator<T: Display + LowerHex + Copy>(
        &self,
        value: i64,
        constants: &Constants,
        directive: &'static str,
        format_width: usize,
        truncate: impl FnOnce(i64) -> (T, bool),
    ) -> impl Iterator<Item = T> {
        let times = self
            .times
            .evaluate(&constants)
            .unwrap_or_else(|_| panic!("{} cannot have forward references", directive))
            as usize; // FIXME: check for negative repeat value

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
        relocation: &mut Vec<RelocationEntry>,
        references: &mut Vec<ReferenceEntry>,
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

                        match symbol.ty {
                            SymbolType::Local | SymbolType::Export => {
                                relocation.push(RelocationEntry::jump(current_offset));
                            }
                            SymbolType::Import => {
                                references.push(ReferenceEntry::jump(symbol, current_offset));
                            }
                        }

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
            Instruction::Pseudo(pseudo_instruction) => pseudo_instruction.lower(
                current_offset,
                constants,
                symbol_table,
                relocation,
                references,
            ),
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

                Self::instructions_to_load_num(value)
            }
            PseudoInstruction::LoadAddress { .. } => 2,
            PseudoInstruction::Move { .. } | PseudoInstruction::Not { .. } => 1,
            PseudoInstruction::Mul { rt, .. }
            | PseudoInstruction::Div { rt, .. }
            | PseudoInstruction::Rem { rt, .. } => match rt {
                Either::Left(_) => 2,
                Either::Right(value) => {
                    let value = value
                        .evaluate(constants)
                        .expect("mul cannot have forward references")
                        as u32;
                    2 + Self::instructions_to_load_num(value)
                }
            },
        }
    }

    pub fn lower(
        self,
        current_offset: usize,
        constants: &Constants,
        symbol_table: &SymbolTable,
        relocation: &mut Vec<RelocationEntry>,
        references: &mut Vec<ReferenceEntry>,
    ) -> Vec<IrInstruction> {
        match self {
            PseudoInstruction::LoadImmediate { rd, value } => {
                let value = value.evaluate(constants).unwrap() as u32;

                Self::load_num_into_register(rd.index().unwrap(), value)
            }
            PseudoInstruction::LoadAddress { rd, label } => {
                let symbol = symbol_table
                    .get(&label)
                    .unwrap_or_else(|| panic!("Could not find symbol '{}'", label));

                match symbol.ty {
                    SymbolType::Local | SymbolType::Export => {
                        relocation.push(RelocationEntry::split_immediate(current_offset));
                    }
                    SymbolType::Import => {
                        references.push(ReferenceEntry::split_immediate(symbol, current_offset));
                    }
                }

                Self::load_u32_into_register(rd.index().unwrap(), symbol.offset as u32)
            }
            PseudoInstruction::Move { rs, rt } => vec![IrInstruction::RType {
                op_code: RTypeOp::Or,
                rs: rs.index().unwrap(),
                rt: 0,
                rd: rt.index().unwrap(),
                shift: 0,
            }],
            PseudoInstruction::Mul { rd, rs, rt } => {
                Self::multiplicative_op(RTypeOp::Mult, RTypeOp::Mflo, constants, rd, rs, rt)
            }
            PseudoInstruction::Div { rd, rs, rt } => {
                Self::multiplicative_op(RTypeOp::Div, RTypeOp::Mflo, constants, rd, rs, rt)
            }
            PseudoInstruction::Rem { rd, rs, rt } => {
                Self::multiplicative_op(RTypeOp::Div, RTypeOp::Mfhi, constants, rd, rs, rt)
            }
            PseudoInstruction::Not { rd, rs } => vec![IrInstruction::RType {
                op_code: RTypeOp::Nor,
                rd: rd.index().unwrap(),
                rs: rs.index().unwrap(),
                rt: 0,
                shift: 0,
            }],
        }
    }

    fn multiplicative_op(
        op_code_1: RTypeOp,
        op_code_2: RTypeOp,
        constants: &Constants,
        rd: Register,
        rs: Register,
        rt: Either<Register, Expr>,
    ) -> Vec<IrInstruction> {
        let (mut instructions, rt) = match rt {
            Either::Left(rt) => (Vec::new(), rt.index().unwrap()),
            Either::Right(value) => {
                let value = value.evaluate(&constants).unwrap() as u32;
                (Self::load_num_into_register(1, value), 1)
            }
        };

        instructions.push(IrInstruction::RType {
            op_code: op_code_1,
            rs: rs.index().unwrap(),
            rt,
            rd: 0,
            shift: 0,
        });
        instructions.push(IrInstruction::RType {
            op_code: op_code_2,
            rs: 0,
            rt: 0,
            rd: rd.index().unwrap(),
            shift: 0,
        });

        instructions
    }

    fn instructions_to_load_num(value: u32) -> usize {
        // We can fit an li into one instruction if the upper or lower
        // 16 bits are zero.
        if value >> 16 & 0xFFFF == 0 || value & 0xFFFF == 0 {
            1
        } else {
            2
        }
    }

    fn load_num_into_register(register: u8, value: u32) -> Vec<IrInstruction> {
        if value >> 16 & 0xFFFF == 0 {
            // If the upper 16 bits are zero, we only need to load the
            // lower bits
            vec![IrInstruction::IType {
                op_code: ITypeOp::Ori,
                rs: 0,
                rt: register,
                immediate: value as i16,
            }]
        } else if value & 0xFFFF == 0 {
            // If the lower 16 bits are zero, we only need to load the
            // upper bits
            vec![IrInstruction::IType {
                op_code: ITypeOp::Lui,
                rs: 0,
                rt: register,
                immediate: (value >> 16) as i16,
            }]
        } else {
            Self::load_u32_into_register(register, value)
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
    /// Calculate the pseudo-address of the symbol from the offset.
    /// (upper four bits and lower two bits removed)
    fn pseudo_address(&self) -> u32 {
        (self.offset as u32 & 0x0FFFFFFC) >> 2
    }
}
