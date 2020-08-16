#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(parser);

mod ast;

fn main() {
    println!("{:?}", parser::ProgramParser::new().parse(".text\n.data"));
}
