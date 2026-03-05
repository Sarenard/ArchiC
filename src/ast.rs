#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub func: Function,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    Return(i64),
}