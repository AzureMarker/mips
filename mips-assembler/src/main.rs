#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(parser);

mod ast;

fn main() {
    println!(
        "{:?}",
        parser::ProgramParser::new()
            .parse("# Test Comment\nMY_CONSTANT=10\n.data\n.text\n.globl main")
    );
}
