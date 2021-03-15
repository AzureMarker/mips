//! Abstract Syntax Tree

use either::Either;

pub type Span = (usize, usize);

#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub data: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(span: Span, data: T) -> Self {
        Spanned { span, data }
    }

    pub fn map<R>(self, f: impl FnOnce(T) -> R) -> Spanned<R> {
        Spanned {
            data: f(self.data),
            span: self.span,
        }
    }
}

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

pub type Expr = Spanned<ExprData>;

impl Expr {
    pub fn calculated(operation: Operation, left: Expr, right: Expr) -> Expr {
        Expr {
            span: (left.span.0, right.span.1),
            data: ExprData::Calculated {
                operation,
                left: Box::new(left),
                right: Box::new(right),
            },
        }
    }
}

impl From<ExprData> for Expr {
    fn from(data: ExprData) -> Self {
        Expr { data, span: (0, 0) }
    }
}

#[derive(Debug)]
pub enum ExprData {
    Number(i64),
    Constant(String),
    Calculated {
        operation: Operation,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Negated(Box<Expr>),
    BitwiseNegated(Box<Expr>),
}

#[derive(Debug)]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
    BitwiseShiftLeft,
    BitwiseShiftRight,
    BitwiseAnd,
    BitwiseXor,
    BitwiseOr,
}

#[derive(Debug)]
pub struct RepeatedExpr {
    pub expr: Expr,
    pub times: Expr,
}

#[derive(Debug)]
pub struct ConstantDef {
    pub name: String,
    pub value: Expr,
}

#[derive(Debug)]
pub enum Directive {
    Text,
    Global {
        label: String,
    },
    Data,
    RData,
    SData,
    Align {
        boundary: Expr,
    },
    Space {
        size: Expr,
    },
    NumberDirective {
        ty: NumberDirective,
        values: Vec<RepeatedExpr>,
    },
    Ascii {
        string: String,
        zero_pad: bool,
    },
}

#[derive(Debug)]
pub enum NumberDirective {
    Byte,
    Half,
    Word,
}

#[derive(Debug)]
pub enum Instruction {
    RType {
        op_code: RTypeOp,
        rd: Register,
        rs: Register,
        rt: Register,
        shift: Expr,
    },
    IType {
        op_code: ITypeOp,
        rs: Register,
        rt: Register,
        immediate: Expr,
    },
    JType {
        op_code: JTypeOp,
        label: Expr,
    },
    Pseudo(PseudoInstruction),
}

impl Instruction {
    /// A no-operation instruction
    pub fn noop() -> Self {
        Instruction::RType {
            op_code: RTypeOp::Sll,
            rd: Register::Number(0),
            rs: Register::Number(0),
            rt: Register::Number(0),
            shift: ExprData::Number(0).into(),
        }
    }
}

#[derive(Debug)]
pub enum PseudoInstruction {
    LoadImmediate {
        rd: Register,
        value: Expr,
    },
    LoadAddress {
        rd: Register,
        label: String,
    },
    Move {
        rt: Register,
        rs: Register,
    },
    Mul {
        rd: Register,
        rs: Register,
        rt: Either<Register, Expr>,
    },
    Div {
        rd: Register,
        rs: Register,
        rt: Either<Register, Expr>,
    },
    Rem {
        rd: Register,
        rs: Register,
        rt: Either<Register, Expr>,
    },
    Not {
        rd: Register,
        rs: Register,
    },
}

#[derive(Debug)]
pub enum RTypeOp {
    Add,
    Addu,
    And,
    Div,
    Divu,
    Jalr,
    Jr,
    Mfhi,
    Mflo,
    Mthi,
    Mtlo,
    Mult,
    Multu,
    Nor,
    Or,
    Sll,
    Sllv,
    Slt,
    Sltu,
    Sra,
    Srav,
    Srl,
    Srlv,
    Sub,
    Subu,
    Syscall,
    Xor,
}

#[derive(Debug)]
pub enum ITypeOp {
    Addi,
    Addiu,
    Andi,
    Bcond,
    Beq,
    Bne,
    Bgtz,
    Blez,
    Lui,
    Lb,
    Lbu,
    Lh,
    Lhu,
    Lw,
    Lwl,
    Lwr,
    Ori,
    Slti,
    Sltiu,
    Sb,
    Sh,
    Sw,
    Swl,
    Swr,
    Xori,
}

impl ITypeOp {
    /// Check if the immediate is an offset. Branching instructions need offsets.
    pub fn needs_offset(&self) -> bool {
        matches!(
            self,
            ITypeOp::Bcond | ITypeOp::Beq | ITypeOp::Bne | ITypeOp::Bgtz | ITypeOp::Blez
        )
    }
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
