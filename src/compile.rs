use std::{
    collections::HashMap, fmt::Write, fs, io::Write as IoWrite, path::PathBuf, process::{Command, Stdio, exit}
};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::{ast::*, parser};

static GLOBAL_LABEL_NB: AtomicU64 = AtomicU64::new(0);

fn new_label() -> u64 {
    GLOBAL_LABEL_NB.fetch_add(1, Ordering::Relaxed)
}

// we only use r0 and r1 in this function
fn compile_expr(out: &mut String, expr : &Expr, variables_table : &HashMap<String, u32>) -> std::fmt::Result {
    match expr {
        Expr::Int(i) => {
            writeln!(out, "push {};", &i)?;
        },
        Expr::Var(name) => {
            // r0 is the addr, r1 is the value
            writeln!(out, "copy r0 {}", variables_table.get(name).unwrap())?;
            writeln!(out, "load r1 [r0]")?;
            writeln!(out, "push r1")?;
        },
        Expr::Add(exprleft, exprright) => {
            compile_expr(out, exprleft, variables_table)?;
            compile_expr(out, exprright, variables_table)?;
            writeln!(out, "pop r0")?;
            writeln!(out, "pop r1")?;
            writeln!(out, "add r0 r0 r1")?;
            writeln!(out, "push r0")?;
        },
        Expr::Sub(exprleft, exprright) => {
            compile_expr(out, exprleft, variables_table)?;
            compile_expr(out, exprright, variables_table)?;
            writeln!(out, "pop r0")?;
            writeln!(out, "pop r1")?;
            writeln!(out, "sub r0 r0 r1")?;
            writeln!(out, "push r0")?;
        }
    }

    Ok(())
}

fn compile_stmt(out: &mut String, stmt: &Stmt, variables_table : &HashMap<String, u32>) -> std::fmt::Result {
    match stmt {
        Stmt::Return(expr) => {
            // let color = if *i == 0 { "green" } else { "red" };
            //writeln!(out, "jump {};", color)?;
            writeln!(out, "; return {:?}", expr)?;
            // we only use r0 and r1 that we restore after
            writeln!(out, "push r0")?;
            writeln!(out, "push r1")?;
            // main work
            compile_expr(out, expr, variables_table)?;
            writeln!(out, "pop r0")?;
            writeln!(out, "skip 1 ifne r0 0")?;
            writeln!(out, "jump green ; true")?;
            writeln!(out, "jump red ; false")?;
            // restoration of registers
            writeln!(out, "pop r1")?;
            writeln!(out, "pop r0")?;
        }
        Stmt::Decl { ty, name, init: expr } => {
            assert!(*ty == Type::U32);
            writeln!(out, "; u32 {} = {:?}", name, expr)?;
            // we only use r0 and r1 that we restore after
            writeln!(out, "push r0")?;
            writeln!(out, "push r1")?;
            // main work
            compile_expr(out, expr, variables_table)?;
            // r0 is the addr, r1 is the value
            writeln!(out, "copy r0 {}", variables_table.get(name).unwrap())?;
            writeln!(out, "pop r1 ; the stack contains the value of {}", name)?;
            writeln!(out, "store [r0] r1")?;
            // restoration of registers
            writeln!(out, "pop r1")?;
            writeln!(out, "pop r0")?;
        }
        Stmt::If { cond, body } => {
            writeln!(out, "; if ({:?})", cond)?;
            // we only use r0 and r1 that we restore after
            writeln!(out, "push r0")?;
            writeln!(out, "push r1")?;
            // main work
            compile_expr(out, cond, variables_table)?;
            let label = new_label();
            // now we have on the stack the if condition, should we jump?
            writeln!(out, "pop r0")?;
            writeln!(out, "skip 1 ifeq r0 0")?;
            writeln!(out, "jump if_{}_true", label)?;
            writeln!(out, "jump if_{}_false", label)?;

            writeln!(out, "if_{}_true:", label)?;
            for smt in body {
                compile_stmt(out, smt, variables_table)?;
            }
            writeln!(out, "jump if_{}_end", label)?;
    
            writeln!(out, "if_{}_false:", label)?; // for else later 
            writeln!(out, "jump if_{}_end", label)?;

            writeln!(out, "if_{}_end:", label)?;
            // restoration of registers
            writeln!(out, "pop r1")?;
            writeln!(out, "pop r0")?;
        },
        Stmt::Assign { name, value } => {
            writeln!(out, "; {} = {:?}", name, value)?;
            // we only use r0 and r1 that we restore after
            writeln!(out, "push r0")?;
            writeln!(out, "push r1")?;
            // main work
            compile_expr(out, value, variables_table)?;
            // r0 is the addr, r1 is the value
            writeln!(out, "copy r0 {}", variables_table.get(name).unwrap())?;
            writeln!(out, "pop r1 ; the stack contains the value of {}", name)?;
            writeln!(out, "store [r0] r1")?;
            // restoration of registers
            writeln!(out, "pop r1")?;
            writeln!(out, "pop r0")?;
        },
        Stmt::While { cond, body } => {
            writeln!(out, "; while ({:?})", cond)?;
            todo!()
        },
    }

    Ok(())
}

pub fn codegen(ast: Program) -> Result<String, String> {

    println!("{:#?}", ast);

    let mut main_func = String::new();

    let mut addr: u32 = 0; // start of the heap
    let mut variables_table: HashMap<String, u32> = HashMap::new();

    // boucle sur les variables
    for stmt in &ast.func.body {
        match stmt {
            Stmt::Decl { ty, name, init: _init } => {
                match ty {
                    Type::Void => {
                        return Err(format!(
                            "Invalid declaration: variable `{}` cannot have type `void`",
                            name
                        ));
                    }
                    Type::U32 => {
                        if variables_table.contains_key(name) {
                            continue;
                        }
                        variables_table.insert(name.to_string(), addr);
                        addr += 4;
                    }
                }
            }
            _ => {}
        }
    }

    println!("{:?}", variables_table);

    match ast {
        Program {
            func: Function { name: _name, body },
        } => {
            writeln!(&mut main_func, "; init of stack").unwrap();
            writeln!(&mut main_func, "let r0 0x00fffffc").unwrap();
            writeln!(&mut main_func, "copy sp r0").unwrap();
            writeln!(&mut main_func, "; main function").unwrap();
            writeln!(&mut main_func, "main:").unwrap();

            for stmt in body {
                let mut line = String::new();
                compile_stmt(&mut line, &stmt, &variables_table).unwrap();

                for l in line.lines() {
                    writeln!(&mut main_func, "    {}", l).unwrap();
                }
            }
        }
    }

    let assembly_output = format!(r#"{}
green:
    copy r3 0
    let r0 0x01000000
    let r1 0x0000FF00
gloop:
    xor r2 r2 r2
    add r2 r2 r3
    add r2 r2 r3
    add r2 r2 r3
    add r2 r2 r3
    store [r0 + r2] r1
    add r3 r3 1
    jump gloop

red:
    copy r3 0
    let r0 0x01000000
    let r1 0x000000FF
rloop:
    xor r2 r2 r2
    add r2 r2 r3
    add r2 r2 r3
    add r2 r2 r3
    add r2 r2 r3
    store [r0 + r2] r1
    add r3 r3 1
    jump rloop
"#, main_func);

    Ok(assembly_output)
}

pub fn bisare(main_ac: PathBuf) -> std::io::Result<()> {
    let src = fs::read_to_string(&main_ac)?;
    let main_asm = main_ac.with_extension("asm");
    let main_bin = main_ac.with_extension("bin");

    let ast = parser::parse_program(&src)
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let codegened = codegen(ast).unwrap();

    fs::write(&main_asm, codegened)?;

    Command::new("python3")
        .arg("bisare/asm.py")
        .arg(&main_asm)
        .status()?;

    let mut child = Command::new("python3")
        .arg("bisare/sim.py")
        .arg(&main_bin)
        .stdin(Stdio::piped())
        .spawn()?;

    if let Some(stdin) = &mut child.stdin {
        stdin.write_all(b"run\n")?;
    }

    let _ = child.wait();

    Ok(())
}