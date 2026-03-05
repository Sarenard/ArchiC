use chumsky::prelude::*;
use std::hash::Hash;

use crate::ast::{Function, Program, Stmt};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Tok {
    Void,
    Ident(String),
    IntLit(i64),
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semi,
    Return,
}

fn lexer() -> impl Parser<char, Vec<Tok>, Error = Simple<char>> {
    let ident = text::ident().map(|s: String| match s.as_str() {
        "void" => Tok::Void,
        "return" => Tok::Return,
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
    ));

    choice((ident, int, punct))
        .padded()
        .repeated()
        .then_ignore(end())
}

fn parser() -> impl Parser<Tok, Program, Error = Simple<Tok>> {
    let ident = select! { Tok::Ident(name) => name };
    let int = select! { Tok::IntLit(n) => n };

    let return_stmt =
        just(Tok::Return)
            .ignore_then(int)
            .then_ignore(just(Tok::Semi))
            .map(Stmt::Return);

    let stmt = return_stmt;

    let block =
        just(Tok::LBrace)
            .ignore_then(stmt.repeated())
            .then_ignore(just(Tok::RBrace));

    just(Tok::Void)
        .ignore_then(ident)
        .then_ignore(just(Tok::LParen))
        .then_ignore(just(Tok::RParen))
        .then(block)
        .then_ignore(end())
        .map(|(name, body)| Program {
            func: Function { name, body },
        })
}

/// Parse un programme mini-C: `int main() {}`
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