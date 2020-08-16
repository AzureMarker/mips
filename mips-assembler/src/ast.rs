#[derive(Debug)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug)]
pub enum Item {
    ConstantDef(ConstantDef),
    Directive(Directive),
    Label(String),
}

#[derive(Debug)]
pub enum Expr {
    Number(i64),
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
}
