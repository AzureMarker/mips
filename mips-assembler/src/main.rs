#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(parser);

mod ast;

fn main() {
    println!(
        "{:?}",
        parser::ProgramParser::new().parse("# Test Comment\n.text\n.data")
    );
}
