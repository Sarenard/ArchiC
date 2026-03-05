use chumsky::prelude::*;
use std::hash::Hash;

use crate::ast::{Expr, Function, Program, Stmt, Type};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Tok {
    Void,
    U32,
    Return,

    Ident(String),
    IntLit(i64),

    LParen,
    RParen,
    LBrace,
    RBrace,
    Semi,
    Eq,
    Plus,
    Minus,
    And,
    LShift,
    RShift,

    If,
    While,
    BinEq,
    BinNEq,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AddOp { 
    Plus, 
    Minus,
    And,
    LShift,
    RShift,
    BinEq,
    BinNEq,
}

fn lexer() -> impl Parser<char, Vec<Tok>, Error = Simple<char>> {
    let ident = text::ident().map(|s: String| match s.as_str() {
        "void" => Tok::Void,
        "return" => Tok::Return,
        "u32" => Tok::U32,
        "if" => Tok::If,
        "while" => Tok::While,
        _ => Tok::Ident(s),
    });

    let int = text::int(10)
        .from_str()
        .unwrapped()
        .map(Tok::IntLit);

    let punct = choice((
        just('(').to(Tok::LParen),
        just(')').to(Tok::RParen),
        just('{').to(Tok::LBrace),
        just('}').to(Tok::RBrace),
        just(';').to(Tok::Semi),
        // before = to not fail
        just("==").to(Tok::BinEq),
        just("!=").to(Tok::BinNEq),
        just('=').to(Tok::Eq),
        just('+').to(Tok::Plus),
        just('-').to(Tok::Minus),
        just('&').to(Tok::And),
        just(">>").to(Tok::RShift),
        just("<<").to(Tok::LShift),
    ));

    choice((ident, int, punct))
        .padded()
        .repeated()
        .then_ignore(end())
}

fn parser() -> impl Parser<Tok, Program, Error = Simple<Tok>> {
    let ident = select! { Tok::Ident(name) => name };
    let int = select! { Tok::IntLit(n) => n };

    // atom = int | ident
    let atom = choice((
        int.map(Expr::Int),
        ident.clone().map(Expr::Var),
    ));

    let add_op = select! {
        Tok::Plus => AddOp::Plus,
        Tok::Minus => AddOp::Minus,
        Tok::And => AddOp::And,
        Tok::RShift => AddOp::RShift,
        Tok::LShift => AddOp::LShift,
        Tok::BinEq => AddOp::BinEq,
        Tok::BinNEq => AddOp::BinNEq,
    };

    let expr = atom.clone()
        .then(add_op.then(atom.clone()).repeated())
        .foldl(|lhs, (op, rhs)| match op {
            AddOp::Plus => Expr::Add(Box::new(lhs), Box::new(rhs)),
            AddOp::Minus => Expr::Sub(Box::new(lhs), Box::new(rhs)),
            AddOp::And => Expr::And(Box::new(lhs), Box::new(rhs)),
            AddOp::LShift => Expr::LShift(Box::new(lhs), Box::new(rhs)),
            AddOp::RShift => Expr::RShift(Box::new(lhs), Box::new(rhs)),
            AddOp::BinEq => Expr::BinEq(Box::new(lhs), Box::new(rhs)),
            AddOp::BinNEq => Expr::BinNEq(Box::new(lhs), Box::new(rhs)),
        });

    let ty = just(Tok::U32).to(Type::U32);

    // u32 x = expr;
    let decl_stmt =
        ty.then(ident.clone())
          .then_ignore(just(Tok::Eq))
          .then(expr.clone())
          .then_ignore(just(Tok::Semi))
          .map(|((ty, name), init)| Stmt::Decl { ty, name, init });

    // x = expr;
    let assign_stmt =
        ident.clone()
        .then_ignore(just(Tok::Eq))
        .then(expr.clone())
        .then_ignore(just(Tok::Semi))
        .map(|(name, value)| Stmt::Assign { name, value });

    let return_stmt =
        just(Tok::Return)
            .ignore_then(expr.clone())
            .then_ignore(just(Tok::Semi))
            .map(Stmt::Return);

    let stmt = recursive(|stmt| {
        let if_stmt =
            just(Tok::If)
                .ignore_then(just(Tok::LParen))
                .ignore_then(expr.clone())
                .then_ignore(just(Tok::RParen))
                .then(
                    just(Tok::LBrace)
                        .ignore_then(stmt.clone().repeated())
                        .then_ignore(just(Tok::RBrace))
                )
                .map(|(cond, body)| Stmt::If { cond, body });

        let while_stmt =
            just(Tok::While)
                .ignore_then(just(Tok::LParen))
                .ignore_then(expr.clone())
                .then_ignore(just(Tok::RParen))
                .then(
                    just(Tok::LBrace)
                        .ignore_then(stmt.clone().repeated())
                        .then_ignore(just(Tok::RBrace))
                )
                .map(|(cond, body)| Stmt::While { cond, body });

        choice((decl_stmt.clone(), assign_stmt, return_stmt.clone(), if_stmt, while_stmt))
    });

    let block =
        just(Tok::LBrace)
            .ignore_then(stmt.repeated());
        
    just(Tok::Void)
        .ignore_then(ident)
        .then_ignore(just(Tok::LParen))
        .then_ignore(just(Tok::RParen))
        .then(block)
        .then_ignore(just(Tok::RBrace))
        .then_ignore(end())
        .map(|(name, body)| Program {
            func: Function { name, body },
        })
}

pub fn parse_program(input: &str) -> Result<Program, String> {
    let tokens = lexer()
        .parse(input)
        .map_err(|errs| format_errors_char("lex", errs))?;

    parser()
        .parse(tokens)
        .map_err(|errs| format_errors_tok("parse", errs))
}

fn format_errors_char(stage: &str, errs: Vec<Simple<char>>) -> String {
    let mut out = String::new();
    out.push_str(&format!("{stage} error(s):\n"));
    for e in errs {
        out.push_str(&format!("  {e}\n"));
    }
    out
}

fn format_errors_tok(stage: &str, errs: Vec<Simple<Tok>>) -> String {
    let mut out = String::new();
    out.push_str(&format!("{stage} error(s):\n"));
    for e in errs {
        out.push_str(&format!("  {e:?}\n"));
    }
    out
}