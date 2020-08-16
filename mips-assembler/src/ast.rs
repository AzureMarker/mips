#[derive(Debug)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug)]
pub enum Item {
    Constant(Constant),
    Directive(Directive),
}

#[derive(Debug)]
pub struct Constant {
    pub name: String,
    pub value: i64,
}

#[derive(Debug)]
pub enum Directive {
    Text,
    Data,
    Global { name: String },
}
