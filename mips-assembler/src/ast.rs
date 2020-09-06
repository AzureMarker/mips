#[derive(Debug)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug)]
pub enum Item {
    ConstantDef(ConstantDef),
    Directive(Directive),
    Label(String),
    Instruction(Instruction),
}

#[derive(Debug)]
pub enum Expr {
    Number(i64),
    Constant(String),
    Calculated {
        operation: Operation,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    // TODO: add unary operations
}

#[derive(Debug)]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
    // TODO: add more operations, like bitwise operations
}

#[derive(Debug)]
pub struct ConstantDef {
    pub name: String,
    pub value: Expr,
}

#[derive(Debug)]
pub enum Directive {
    Text,
    Global { name: String },
    Data,
    Align { boundary: Expr },
    Space { size: Expr },
    Word { count: Expr },
    Asciiz { string: String },
}

#[derive(Debug)]
pub enum Instruction {
    RType {
        op_code: RTypeOp,
        rd: Register,
        rs: Register,
        rt: Register,
    },
    IType {
        op_code: ITypeOp,
        rs: Register,
        rt: Register,
        immediate: Expr,
    },
    JType {
        op_code: JTypeOp,
        label: String,
    },
    Syscall,
    Pseudo(PseudoInstruction),
}

#[derive(Debug)]
pub enum PseudoInstruction {
    LoadImmediate { rd: Register, value: Expr },
    LoadAddress { rd: Register, label: String },
    Move { rt: Register, rs: Register },
}

#[derive(Debug)]
pub enum RTypeOp {
    Add,
    Jr,
}

#[derive(Debug)]
pub enum ITypeOp {
    Addi,
    Beq,
    Sw,
    Lw,
}

#[derive(Debug)]
pub enum JTypeOp {
    Jump,
    Jal,
}

#[derive(Debug)]
pub enum Register {
    Number(u8),
    Name(String),
}

impl Register {
    /// Get the register index. If the register is invalid, None is returned.
    pub fn index(&self) -> Option<u8> {
        match self {
            Register::Number(num) => Some(*num).filter(|num| *num < 32),
            Register::Name(name) => match name.as_str() {
                "zero" => Some(0),
                "at" => Some(1),
                "v0" => Some(2),
                "v1" => Some(3),
                "a0" => Some(4),
                "a1" => Some(5),
                "a2" => Some(6),
                "a3" => Some(7),
                "t0" => Some(8),
                "t1" => Some(9),
                "t2" => Some(10),
                "t3" => Some(11),
                "t4" => Some(12),
                "t5" => Some(13),
                "t6" => Some(14),
                "t7" => Some(15),
                "s0" => Some(16),
                "s1" => Some(17),
                "s2" => Some(18),
                "s3" => Some(19),
                "s4" => Some(20),
                "s5" => Some(21),
                "s6" => Some(22),
                "s7" => Some(23),
                "t8" => Some(24),
                "t9" => Some(25),
                "k0" => Some(26),
                "k1" => Some(27),
                "gp" => Some(28),
                "sp" => Some(29),
                "fp" => Some(30),
                "ra" => Some(31),
                _ => None,
            },
        }
    }
}
