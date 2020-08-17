#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(parser);

mod ast;

fn main() {
    println!(
        "{:#?}",
        parser::ProgramParser::new().parse(
            "# Test Comment\n\
            MY_CONSTANT=1+1\n\
            .data\n\
            .text\n\
            .globl main\n\
            main:\n\
                add $t1 $zero $t0\n"
        )
    );
}
