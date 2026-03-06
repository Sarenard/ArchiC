use chumsky::prelude::*;
use std::hash::Hash;

use crate::ast::{BaseType, Expr, Function, Program, Stmt, Type};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Tok {
    Void,
    U32,
    Return,

    Ident(String),
    StrLit(String),
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
    Or,
    LE,
    GE,
    GT,
    LT,
    Comma,
    Star,
    AddrOf,
    For,
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
    Or,
    LE,
    GE,
    GT,
    LT,
}

fn lexer() -> impl Parser<char, Vec<Tok>, Error = Simple<char>> {
    let comment = just("//")
        .ignore_then(filter(|c: &char| *c != '\n').repeated())
        .ignored();

    let ws = one_of(" \t\r\n").repeated().at_least(1).ignored();

    let ignored = choice((ws, comment)).repeated();

    let ident = text::ident().map(|s: String| match s.as_str() {
        "void" => Tok::Void,
        "return" => Tok::Return,
        "u32" => Tok::U32,
        "if" => Tok::If,
        "while" => Tok::While,
        "for" => Tok::For,
        _ => Tok::Ident(s),
    });

    let hex = just("0x")
        .or(just("0X"))
        .ignore_then(
            filter(|c: &char| c.is_ascii_hexdigit())
                .repeated()
                .at_least(1)
                .collect::<String>(),
        )
        .map(|s| Tok::IntLit(i64::from_str_radix(&s, 16).unwrap()));

    let dec = text::int(10)
        .from_str()
        .unwrapped()
        .map(Tok::IntLit);

    let escaped_char = just('\\').ignore_then(choice((
        just('n').to('\n'),
        just('r').to('\r'),
        just('t').to('\t'),
        just('0').to('\0'),
        just('\\').to('\\'),
        just('\'').to('\''),
        just('"').to('"'),
    )));

    let char_lit = just('\'')
        .ignore_then(choice((
            escaped_char.clone(),
            filter(|c: &char| *c != '\'' && *c != '\n' && *c != '\\'),
        )))
        .then_ignore(just('\''))
        .map(|c| Tok::IntLit(c as i64));

    let string_lit = just('"')
        .ignore_then(
            choice((
                escaped_char.map(|c| c.to_string()),
                filter(|c: &char| *c != '"' && *c != '\n' && *c != '\\')
                    .map(|c| c.to_string()),
            ))
            .repeated()
            .collect::<Vec<_>>()
            .map(|parts| parts.concat()),
        )
        .then_ignore(just('"'))
        .map(Tok::StrLit);

    let int = choice((hex, dec, char_lit));

    let punct = choice((
        just("==").to(Tok::BinEq),
        just("!=").to(Tok::BinNEq),
        just("<<").to(Tok::LShift),
        just(">>").to(Tok::RShift),
        just("<=").to(Tok::LE),
        just(">=").to(Tok::GE),
        just("&&").to(Tok::And),
        just("||").to(Tok::Or),

        just('(').to(Tok::LParen),
        just(')').to(Tok::RParen),
        just('{').to(Tok::LBrace),
        just('}').to(Tok::RBrace),
        just(';').to(Tok::Semi),
        just(',').to(Tok::Comma),
        just('*').to(Tok::Star),

        just('<').to(Tok::LT),
        just('>').to(Tok::GT),
        just('=').to(Tok::Eq),

        just('+').to(Tok::Plus),
        just('-').to(Tok::Minus),
        just('&').to(Tok::AddrOf),
    ));

    let token = choice((ident, string_lit, int, punct));

    ignored.clone()
        .ignore_then(token)
        .then_ignore(ignored)
        .repeated()
        .then_ignore(end())
}

fn parser() -> impl Parser<Tok, Program, Error = Simple<Tok>> {
    let ident = select! { Tok::Ident(name) => name };
    
    let int = select! { Tok::IntLit(n) => n };

    let string = select! { Tok::StrLit(s) => s };

    let add_op = select! {
        Tok::Plus => AddOp::Plus,
        Tok::Minus => AddOp::Minus,
        Tok::And => AddOp::And,
        Tok::RShift => AddOp::RShift,
        Tok::LShift => AddOp::LShift,
        Tok::BinEq => AddOp::BinEq,
        Tok::BinNEq => AddOp::BinNEq,
        Tok::Or => AddOp::Or,
        Tok::LE => AddOp::LE,
        Tok::GE => AddOp::GE,
        Tok::GT => AddOp::GT,
        Tok::LT => AddOp::LT,
    };

    let expr = recursive(|expr| {
        // (expr)
        let paren = just(Tok::LParen)
            .ignore_then(expr.clone())
            .then_ignore(just(Tok::RParen));

        // args = expr (',' expr)* | vide
        let args = expr
            .clone()
            .separated_by(just(Tok::Comma))
            .allow_trailing()
            .or_not()
            .map(|opt| opt.unwrap_or_default());

        // call = Ident '(' args ')'
        let call = ident
            .clone()
            .then_ignore(just(Tok::LParen))
            .then(args)
            .then_ignore(just(Tok::RParen))
            .map(|(name, args)| Expr::Call { name, args });

        // primary = call | int | var | (expr)
        let primary = choice((
            call,
            string.map(Expr::Str),
            int.map(Expr::Int),
            ident.clone().map(Expr::Var),
            paren,
        ));

        // unary = ('&' unary) | ('*' unary) | primary
        let unary = recursive(|unary| {
            choice((
                just(Tok::AddrOf)
                    .ignore_then(unary.clone())
                    .map(|e| Expr::AddrOf(Box::new(e))),
                just(Tok::Star)
                    .ignore_then(unary.clone())
                    .map(|e| Expr::Deref(Box::new(e))),
                primary.clone(),
            ))
        });

        // binary
        unary
            .clone()
            .then(add_op.then(unary).repeated())
            .foldl(|lhs, (op, rhs)| match op {
                AddOp::Plus => Expr::Add(Box::new(lhs), Box::new(rhs)),
                AddOp::Minus => Expr::Sub(Box::new(lhs), Box::new(rhs)),
                AddOp::And => Expr::And(Box::new(lhs), Box::new(rhs)), // binaire &
                AddOp::Or => Expr::Or(Box::new(lhs), Box::new(rhs)),
                AddOp::LShift => Expr::LShift(Box::new(lhs), Box::new(rhs)),
                AddOp::RShift => Expr::RShift(Box::new(lhs), Box::new(rhs)),
                AddOp::BinEq => Expr::BinEq(Box::new(lhs), Box::new(rhs)),
                AddOp::BinNEq => Expr::BinNEq(Box::new(lhs), Box::new(rhs)),
                AddOp::LE => Expr::LE(Box::new(lhs), Box::new(rhs)),
                AddOp::GE => Expr::GE(Box::new(lhs), Box::new(rhs)),
                AddOp::LT => Expr::LT(Box::new(lhs), Box::new(rhs)),
                AddOp::GT => Expr::GT(Box::new(lhs), Box::new(rhs)),
            })
    });

    let base_ty = choice((
        just(Tok::Void).to(BaseType::Void),
        just(Tok::U32).to(BaseType::U32),
    ));

    let ty = base_ty
        .then(just(Tok::Star).repeated())
        .map(|(base, stars)| Type { base, ptr: stars.len() as u32 });

    // u32 x = expr;
    let decl_stmt =
        ty.clone()
        .then(ident.clone())
        .then_ignore(just(Tok::Eq))
        .then(expr.clone())
        .then_ignore(just(Tok::Semi))
        .map(|((ty, name), init)| Stmt::Decl { ty, name, init });

    let lvalue =
        choice((
            ident.clone().map(Expr::Var),
            just(Tok::Star)
                .ignore_then(expr.clone())
                .map(|e| Expr::Deref(Box::new(e))),
        ));

    // x = expr;
    let assign_stmt =
        lvalue.clone()
        .then_ignore(just(Tok::Eq))
        .then(expr.clone())
        .then_ignore(just(Tok::Semi))
        .map(|(target, value)| Stmt::Assign { target, value });

    // u32 x = expr   (sans ; )
    let decl_no_semi =
        ty.clone()
        .then(ident.clone())
        .then_ignore(just(Tok::Eq))
        .then(expr.clone())
        .map(|((ty, name), init)| Stmt::Decl { ty, name, init });

    // lvalue = expr   (sans ; )
    let assign_no_semi =
        lvalue.clone()
        .then_ignore(just(Tok::Eq))
        .then(expr.clone())
        .map(|(target, value)| Stmt::Assign { target, value });

    let return_stmt =
        just(Tok::Return)
        .ignore_then(expr.clone())
        .then_ignore(just(Tok::Semi))
        .map(Stmt::Return);

    let expr_stmt =
        expr.clone()
        .then_ignore(just(Tok::Semi))
        .map(Stmt::Expr);

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

        let for_stmt =
            just(Tok::For)
            .ignore_then(just(Tok::LParen))
            .ignore_then(
                // init: decl | assign | (vide)
                choice((
                    decl_no_semi.clone().map(|s| Some(Box::new(s))),
                    assign_no_semi.clone().map(|s| Some(Box::new(s))),
                    empty().to(None),
                ))
            )
            .then_ignore(just(Tok::Semi))
            .then(
                // cond: expr | vide
                expr.clone().or_not()
            )
            .then_ignore(just(Tok::Semi))
            .then(
                // step: assign | expr-stmt (optionnel) | vide
                choice((
                    assign_no_semi.clone().map(|s| Some(Box::new(s))),
                    expr.clone().map(Stmt::Expr).map(|s| Some(Box::new(s))),
                    empty().to(None),
                ))
            )
            .then_ignore(just(Tok::RParen))
            .then(
                just(Tok::LBrace)
                    .ignore_then(stmt.clone().repeated())
                    .then_ignore(just(Tok::RBrace))
            )
            .map(|(((init, cond), step), body)| Stmt::For { init, cond, step, body });

        choice((decl_stmt.clone(), assign_stmt, return_stmt.clone(), if_stmt, while_stmt, for_stmt, expr_stmt.clone()))
    });

    let block =
        just(Tok::LBrace)
        .ignore_then(stmt.repeated())
        .then_ignore(just(Tok::RBrace));

    let param =
        ty.clone()
        .then(ident.clone())
        .map(|(ty, name)| (ty, name));

    let params =
        param
        .separated_by(just(Tok::Comma))
        .allow_trailing()
        .or_not()
        .map(|p| p.unwrap_or_default());
        
    let function =
        ty
        .then(ident.clone())
        .then_ignore(just(Tok::LParen))
        .then(params)
        .then_ignore(just(Tok::RParen))
        .then(block)
        .map(|(((return_ty, name), params), body)| Function {
            return_ty,
            name,
            params,
            body,
        });

    function
        .repeated()
        .at_least(1)          // optionnel: exiger au moins 1 fonction
        .then_ignore(end())
        .map(|funcs| Program { funcs })
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