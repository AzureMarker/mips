//! Lower the AST to HIR

use crate::ast::{ConstantDef, Directive, Expr, Instruction, Item, Operation, Program};
use crate::hir::{
    HirData, HirInstruction, HirProgram, HirText, Symbol, SymbolLocation, SymbolTable,
};
use std::collections::HashMap;

impl Program {
    pub fn lower(self) -> HirProgram {
        let mut instructions: Vec<HirInstruction> = Vec::new();
        let mut data: Vec<u8> = Vec::new();
        let mut symbol_table: HashMap<String, Symbol> = HashMap::new();
        let mut globals: Vec<String> = Vec::new();
        let mut constants: HashMap<String, i64> = HashMap::new();
        let mut current_section = SymbolLocation::Text;

        for item in self.items {
            match item {
                Item::ConstantDef(ConstantDef { name, value }) => {
                    // TODO: create a dependency graph of constants?
                    constants.insert(name, value.evaluate(&constants));
                }
                Item::Label(name) => {
                    symbol_table.insert(
                        name,
                        Symbol {
                            location: current_section,
                            offset: match current_section {
                                SymbolLocation::Text => instructions.len() * 4,
                                SymbolLocation::Data => data.len(),
                            },
                        },
                    );
                }
                Item::Directive(directive) => match directive {
                    Directive::Text => current_section = SymbolLocation::Text,
                    Directive::Data => current_section = SymbolLocation::Data,
                    Directive::Global { label } => globals.push(label),
                    Directive::Align { boundary } => {
                        let boundary = boundary.evaluate(&constants) as usize;
                        data.extend(std::iter::repeat(0).take(boundary - (data.len() % boundary)));
                    }
                    Directive::Space { size } => {
                        data.extend(std::iter::repeat(0).take(size.evaluate(&constants) as usize))
                    }
                    Directive::Word { values } => data.extend(
                        values
                            .into_iter()
                            .flat_map(|e| e.evaluate(&constants).to_be_bytes().to_vec()),
                    ),
                    Directive::Asciiz { string } => {
                        // TODO: enforce only ASCII?
                        data.extend(string.bytes().chain(std::iter::once(0)))
                    }
                },
                Item::Instruction(instruction) => instructions.push(instruction.lower()),
            }
        }

        HirProgram {
            text: HirText { instructions },
            data: HirData { data },
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
            Expr::Negated(expr) => -1 * expr.evaluate(constants),
        }
    }
}

impl Instruction {
    pub fn lower(self) -> HirInstruction {
        unimplemented!()
    }
}
