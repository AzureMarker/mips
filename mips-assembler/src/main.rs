#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(parser);

fn main() {
    println!("{:?}", parser::ItemParser::new().parse(".text"));
}
