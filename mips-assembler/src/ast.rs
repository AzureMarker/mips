#[derive(Debug)]
pub struct Program<'input> {
    pub items: Vec<Item<'input>>,
}

#[derive(Debug)]
pub enum Item<'input> {
    Constant(Constant<'input>),
    Directive(Directive<'input>),
}

#[derive(Debug)]
pub struct Constant<'input> {
    pub name: &'input str,
    pub value: i64,
}

#[derive(Debug)]
pub struct Directive<'input>(pub &'input str);
