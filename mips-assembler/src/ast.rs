#[derive(Debug)]
pub struct Program<'input> {
    pub items: Vec<Item<'input>>,
}

#[derive(Debug)]
pub enum Item<'input> {
    Directive(Directive<'input>),
}

#[derive(Debug)]
pub struct Directive<'input>(pub &'input str);
