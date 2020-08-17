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
            .space MY_CONSTANT*4\n\
            .word 2\n\
            .text\n\
            .globl main\n\
            main:\n\
                li $t0, 1		# $t0 = 1\n\
                li $t1, 10		# $t1 = 10\n\
                add $t2, $t0, $t1	# $t2 = $t0 + $t1\n"
        )
    );
}
