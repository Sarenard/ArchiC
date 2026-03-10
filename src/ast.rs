#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub funcs: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub return_ty: Type,
    pub name: String,
    pub params: Vec<(Type, String)>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaseType { Void, U32 }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type {
    pub base: BaseType,
    pub ptr: u32, // nombre de '*'
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Int(i64),
    Var(String),
    Call { name : String, args: Vec<Expr> },

    AddrOf(Box<Expr>),
    Deref(Box<Expr>),

    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    LShift(Box<Expr>, Box<Expr>),
    RShift(Box<Expr>, Box<Expr>),
    BinEq(Box<Expr>, Box<Expr>),
    BinNEq(Box<Expr>, Box<Expr>),
    LE(Box<Expr>, Box<Expr>),
    GE(Box<Expr>, Box<Expr>),
    GT(Box<Expr>, Box<Expr>),
    LT(Box<Expr>, Box<Expr>),

    Str(String),
    ArrayLit(Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    Return(Expr),
    Decl { ty: Type, name: String, init: Expr },
    Assign { target: Expr, value: Expr },
    If { cond: Expr, body: Vec<Stmt> },
    While { cond: Expr, body: Vec<Stmt> },
    For {
        init: Option<Box<Stmt>>,
        cond: Option<Expr>,
        step: Option<Box<Stmt>>,
        body: Vec<Stmt>,
    },

    Expr(Expr),
}