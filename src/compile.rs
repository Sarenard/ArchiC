use std::{
    collections::HashMap, fmt::Write, fs, io::Write as IoWrite, path::PathBuf, process::{Command, Stdio}
};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::{ast::*, parser};

static GLOBAL_LABEL_NB: AtomicU64 = AtomicU64::new(0);

fn new_label() -> u64 {
    GLOBAL_LABEL_NB.fetch_add(1, Ordering::Relaxed)
}

// we only use r0 and r1 in this function
fn compile_expr(out: &mut String, expr : &Expr, fun_name: &String, variables_table : &HashMap<(String, String), u32>) -> std::fmt::Result {
    match expr {
        Expr::Int(i) => {
            // if the val is too big we need to be a little smarter
            writeln!(out, "let r0 {}", &i)?;
            writeln!(out, "push r0")?;
        },
        Expr::Var(name) => {
            // r0 is the addr, r1 is the value
            writeln!(out, "copy r0 {}", variables_table.get(&(fun_name.clone(), name.clone())).unwrap())?;
            writeln!(out, "load r1 [r0]")?;
            writeln!(out, "push r1")?;
        },
        Expr::Add(exprleft, exprright) => {
            compile_expr(out, exprleft, fun_name, variables_table)?;
            compile_expr(out, exprright, fun_name, variables_table)?;
            writeln!(out, "pop r1")?;
            writeln!(out, "pop r0")?;
            writeln!(out, "add r0 r0 r1")?;
            writeln!(out, "push r0")?;
        },
        Expr::Sub(exprleft, exprright) => {
            compile_expr(out, exprleft, fun_name, variables_table)?;
            compile_expr(out, exprright, fun_name, variables_table)?;
            writeln!(out, "pop r1")?;
            writeln!(out, "pop r0")?;
            writeln!(out, "sub r0 r0 r1")?;
            writeln!(out, "push r0")?;
        }
        Expr::And(exprleft, exprright) => {
            compile_expr(out, exprleft, fun_name, variables_table)?;
            compile_expr(out, exprright, fun_name, variables_table)?;
            writeln!(out, "pop r1")?;
            writeln!(out, "pop r0")?;
            writeln!(out, "and r0 r0 r1")?;
            writeln!(out, "push r0")?;
        },
        Expr::LShift(exprleft, exprright) => {
            compile_expr(out, exprleft, fun_name, variables_table)?;
            compile_expr(out, exprright, fun_name, variables_table)?;
            writeln!(out, "pop r1")?;
            writeln!(out, "pop r0")?;
            writeln!(out, "lsl r0 r0 r1")?;
            writeln!(out, "push r0")?;
        },
        Expr::RShift(exprleft, exprright) => {
            compile_expr(out, exprleft, fun_name, variables_table)?;
            compile_expr(out, exprright, fun_name, variables_table)?;
            writeln!(out, "pop r1")?;
            writeln!(out, "pop r0")?;
            writeln!(out, "lsr r0 r0 r1")?;
            writeln!(out, "push r0")?;
        },
        Expr::BinEq(exprleft, exprright) => {
            compile_expr(out, exprleft, fun_name, variables_table)?;
            compile_expr(out, exprright, fun_name, variables_table)?;
            writeln!(out, "pop r1")?;
            writeln!(out, "pop r0")?;
            let label = new_label();
            writeln!(out, "skip 1 ifeq r0 r1")?;
            writeln!(out, "jump eq_{}_false", label)?;
            writeln!(out, "jump eq_{}_true", label)?;
    
            writeln!(out, "eq_{}_true:", label)?;
            writeln!(out, "push 1")?;
            writeln!(out, "jump eq_{}_end", label)?;
    
            writeln!(out, "eq_{}_false:", label)?;
            writeln!(out, "push 0")?;
            writeln!(out, "jump eq_{}_end", label)?;

            writeln!(out, "eq_{}_end:", label)?;
        },
        Expr::Or(exprleft, exprright) => {
            compile_expr(out, exprleft, fun_name, variables_table)?;
            compile_expr(out, exprright, fun_name, variables_table)?;
            writeln!(out, "pop r1")?;
            writeln!(out, "pop r0")?;
            writeln!(out, "or r0 r0 r1")?;
            writeln!(out, "push r0")?;
        },
        Expr::BinNEq(exprleft, exprright) => {
            compile_expr(out, exprleft, fun_name, variables_table)?;
            compile_expr(out, exprright, fun_name, variables_table)?;
            writeln!(out, "pop r1")?;
            writeln!(out, "pop r0")?;
            let label = new_label();
            writeln!(out, "skip 1 ifne r0 r1")?;
            writeln!(out, "jump eq_{}_false", label)?;
            writeln!(out, "jump eq_{}_true", label)?;
    
            writeln!(out, "eq_{}_true:", label)?;
            writeln!(out, "push 1")?;
            writeln!(out, "jump eq_{}_end", label)?;
    
            writeln!(out, "eq_{}_false:", label)?;
            writeln!(out, "push 0")?;
            writeln!(out, "jump eq_{}_end", label)?;

            writeln!(out, "eq_{}_end:", label)?;
        },
        Expr::LE(exprleft, exprright) => {
            compile_expr(out, exprleft, fun_name, variables_table)?;
            compile_expr(out, exprright, fun_name, variables_table)?;
            writeln!(out, "pop r1")?;
            writeln!(out, "pop r0")?;
            let label = new_label();
            writeln!(out, "skip 1 ifle r0 r1")?;
            writeln!(out, "jump eq_{}_false", label)?;
            writeln!(out, "jump eq_{}_true", label)?;
    
            writeln!(out, "eq_{}_true:", label)?;
            writeln!(out, "push 1")?;
            writeln!(out, "jump eq_{}_end", label)?;
    
            writeln!(out, "eq_{}_false:", label)?;
            writeln!(out, "push 0")?;
            writeln!(out, "jump eq_{}_end", label)?;

            writeln!(out, "eq_{}_end:", label)?;
        },
        Expr::GE(exprleft, exprright) => {
            compile_expr(out, exprleft, fun_name, variables_table)?;
            compile_expr(out, exprright, fun_name, variables_table)?;
            writeln!(out, "pop r1")?;
            writeln!(out, "pop r0")?;
            let label = new_label();
            writeln!(out, "skip 1 ifge r0 r1")?;
            writeln!(out, "jump eq_{}_false", label)?;
            writeln!(out, "jump eq_{}_true", label)?;
    
            writeln!(out, "eq_{}_true:", label)?;
            writeln!(out, "push 1")?;
            writeln!(out, "jump eq_{}_end", label)?;
    
            writeln!(out, "eq_{}_false:", label)?;
            writeln!(out, "push 0")?;
            writeln!(out, "jump eq_{}_end", label)?;

            writeln!(out, "eq_{}_end:", label)?;
        },
        Expr::GT(exprleft, exprright) => {
            compile_expr(out, exprleft, fun_name, variables_table)?;
            compile_expr(out, exprright, fun_name, variables_table)?;
            writeln!(out, "pop r1")?;
            writeln!(out, "pop r0")?;
            let label = new_label();
            writeln!(out, "skip 1 ifgt r0 r1")?;
            writeln!(out, "jump eq_{}_false", label)?;
            writeln!(out, "jump eq_{}_true", label)?;
    
            writeln!(out, "eq_{}_true:", label)?;
            writeln!(out, "push 1")?;
            writeln!(out, "jump eq_{}_end", label)?;
    
            writeln!(out, "eq_{}_false:", label)?;
            writeln!(out, "push 0")?;
            writeln!(out, "jump eq_{}_end", label)?;

            writeln!(out, "eq_{}_end:", label)?;
        },
        Expr::LT(exprleft, exprright) => {
            compile_expr(out, exprleft, fun_name, variables_table)?;
            compile_expr(out, exprright, fun_name, variables_table)?;
            writeln!(out, "pop r1")?;
            writeln!(out, "pop r0")?;
            let label = new_label();
            writeln!(out, "skip 1 iflt r0 r1")?;
            writeln!(out, "jump eq_{}_false", label)?;
            writeln!(out, "jump eq_{}_true", label)?;
    
            writeln!(out, "eq_{}_true:", label)?;
            writeln!(out, "push 1")?;
            writeln!(out, "jump eq_{}_end", label)?;
    
            writeln!(out, "eq_{}_false:", label)?;
            writeln!(out, "push 0")?;
            writeln!(out, "jump eq_{}_end", label)?;

            writeln!(out, "eq_{}_end:", label)?;
        },
        Expr::Call { name, args } => {
            /* 
            Calling convention
            The caller pushes the arguments left to right
            Then calls the function
            The function then unfolds the arguments into the variables
            Then proceeds as normal
            Example :
                void f(x, y) {

                }
                void main() {
                    f(1, 2)
                }
            Becomes (cleaned up of the bullshit of course):
                main:
                    push 1
                    push 2
                    jump f ; pushes the adress to the stack
                    (*continuation of main*)
                f:
                    swap
                    x := pop
                    swap
                    y := pop
                    (*body*)
                    ret
            */
            // push of args
            for arg in args.iter().rev() {
                compile_expr(out, arg, fun_name, variables_table)?;
            }
            // call
            writeln!(out, "call {}", name)?;
        },
        Expr::AddrOf(expr) => {
            match expr.as_ref() {
                Expr::Var(name) => {
                    let addr = variables_table.get(&(fun_name.clone(), name.clone())).unwrap();
                    writeln!(out, "push {}", addr)?;
                },
                Expr::Deref(expr) => {
                    // addrof \circ deref = id, we pass as if nothing happened
                    // TODO : check if expr is a pointer (else we should probably fail)
                    compile_expr(out, expr, fun_name, variables_table)?;
                },
                _ => panic!("Error : cannot get adress of {:?}", expr.as_ref()),
            }
        },
        Expr::Deref(expr) => {
            compile_expr(out, expr, fun_name, variables_table)?;
            // now we need to push the value at 
            writeln!(out, "pop r0")?;
            writeln!(out, "load r1 [r0]")?;
            writeln!(out, "push r1")?;
        },
    }

    Ok(())
}

fn compile_stmt(out: &mut String, stmt: &Stmt, fun_name: &String, variables_table : &HashMap<(String, String), u32>) -> std::fmt::Result {
    match stmt {
        Stmt::Return(expr) => {
            // we add the return value below the ret addr that is at the top of the stack
            writeln!(out, "; return {:?}", expr)?;
            compile_expr(out, expr, fun_name, variables_table)?;
            writeln!(out, "pop r0")?;
            writeln!(out, "pop r1")?;
            writeln!(out, "push r0")?;
            writeln!(out, "push r1")?;
            writeln!(out, "ret")?;
        }
        Stmt::Decl { ty, name, init: expr } => {
            assert!(!(ty.base == BaseType::Void && ty.ptr == 0)); // we cant assign to void
            writeln!(out, "; u32 {} = {:?}", name, expr)?;
            // main work
            compile_expr(out, expr, fun_name, variables_table)?;
            // r0 is the addr, r1 is the value
            writeln!(out, "copy r0 {}", variables_table.get(&(fun_name.clone(), name.clone())).unwrap())?;
            writeln!(out, "pop r1 ; the stack contains the value of {}", name)?;
            writeln!(out, "store [r0] r1")?;
        }
        Stmt::If { cond, body } => {
            writeln!(out, "; if ({:?})", cond)?;
            // main work
            compile_expr(out, cond, fun_name, variables_table)?;
            let label = new_label();
            // now we have on the stack the if condition, where should we jump?
            writeln!(out, "pop r0")?;
            writeln!(out, "skip 1 ifeq r0 0")?;
            writeln!(out, "jump if_{}_true", label)?;
            writeln!(out, "jump if_{}_false", label)?;

            writeln!(out, "if_{}_true:", label)?;
            for smt in body {
                compile_stmt(out, smt, fun_name, variables_table)?;
            }
            writeln!(out, "jump if_{}_end", label)?;
    
            writeln!(out, "if_{}_false:", label)?; // for else later 
            writeln!(out, "jump if_{}_end", label)?;

            writeln!(out, "if_{}_end:", label)?;
        },
        Stmt::Assign { target, value } => {
            // TODO (typing) : check that the right side is not void?
            
            fn get_addr(
                out: &mut String,
                expr: &Expr,
                variables_table: &HashMap<(String, String), u32>,
                fun_name: &str,
            ) -> std::fmt::Result {
                match expr {
                    Expr::Var(name) => {
                        let addr = *variables_table
                            .get(&(fun_name.to_string(), name.clone()))
                            .unwrap();
                        writeln!(out, "copy r0 {}", addr)?;
                    }

                    Expr::Deref(inner) => {
                        get_addr(out, inner, variables_table, fun_name)?;
                        writeln!(out, "load r0 [r0]")?;
                    }

                    _ => panic!("not an lvalue"),
                }

                Ok(())
            }
            
            writeln!(out, "; {:?} = {:?}", target, value)?;
            // main work
            compile_expr(out, value, fun_name, variables_table)?;
            // r0 is the addr, r1 is the value
            writeln!(out, "pop r1 ; the stack contains the value of {:?}", target)?;
            // we compute where to write (and put it in r0)
            get_addr(out, target, &variables_table, &fun_name)?;
            writeln!(out, "store [r0] r1")?;
        },
        Stmt::While { cond, body } => {
            writeln!(out, "; while ({:?})", cond)?;
            // main work
            let label = new_label();
            writeln!(out, "while_{}_check:", label)?;
            compile_expr(out, cond, fun_name, variables_table)?;
            // now we have on the stack the while condition, where should we jump?
            writeln!(out, "pop r0")?;
            writeln!(out, "skip 1 ifeq r0 0")?;
            writeln!(out, "jump while_{}_true", label)?;
            writeln!(out, "jump while_{}_end", label)?;

            writeln!(out, "while_{}_true:", label)?;
            // content of the while
            for smt in body {
                compile_stmt(out, smt, fun_name, variables_table)?;
            }
            writeln!(out, "jump while_{}_check", label)?;

            writeln!(out, "while_{}_end:", label)?;
        },
        Stmt::Expr(expr) => {
            writeln!(out, "; raw expr ({:?})", expr)?;
            compile_expr(out, expr, fun_name, variables_table)?;
            // TODO : dont do that if statement is void function
            writeln!(out, "pop r0 ; pop to not grow the stack")?;
        },
    }

    Ok(())
}

pub fn codegen(ast: Program) -> Result<String, String> {

    println!("{:#?}", ast);

    let mut main_func = String::new();

    // after the program, before the stack, before the zone that is annoying to parse
    let mut addr: u32 = 0x0000EFFC;
    let mut variables_table: HashMap<(String, String), u32> = HashMap::new();

    fn ensure_not_void(ty: &Type, name: &str) -> Result<(), String> {
        match ty.base {
            BaseType::Void => Err(format!("Invalid declaration: variable `{}` cannot have type `void`", name)),
            BaseType::U32 => Ok(()),
        }
    }

    fn declare_var(
        variables_table: &mut HashMap<(String, String), u32>,
        func_name: &str,
        var_name: &str,
        ty: &Type,
        addr: &mut u32,
    ) -> Result<(), String> {
        ensure_not_void(ty, var_name)?;

        let key = (func_name.to_string(), var_name.to_string());
        if variables_table.contains_key(&key) {
            return Err(format!("Invalid declaration: variable `{}` already exists", var_name));
        }

        variables_table.insert(key, *addr);
        *addr += 4;
        Ok(())
    }

    for func in &ast.funcs {
        // params
        for (param_ty, param_name) in &func.params {
            declare_var(
                &mut variables_table,
                &func.name,
                param_name,
                param_ty,
                &mut addr,
            )?;
        }

        // decls dans le body
        for stmt in &func.body {
            match stmt {
                Stmt::Decl { ty, name, .. } => {
                    declare_var(
                        &mut variables_table,
                        &func.name,
                        name,
                        ty,
                        &mut addr,
                    )?;
                }
                _ => {}
            }
        }
    }

    println!("{:?}", variables_table);

    for func in &ast.funcs {
        writeln!(&mut main_func, "{}:", &func.name).unwrap();

        let mut params = String::new();
        writeln!(&mut params, "; params handling").unwrap();
        for (param_ty, param_name) in &func.params {
            // see the calling convention documented in the Call expr
            ensure_not_void(param_ty, param_name)?;
            writeln!(&mut params, "; set of {}", param_name).unwrap();
            writeln!(&mut params, "pop r0").unwrap();
            writeln!(&mut params, "pop r1").unwrap();
            writeln!(&mut params, "push r0").unwrap();
            // r1 is now our argument
            writeln!(&mut params, "copy r0 {}", variables_table.get(&(func.name.clone(), param_name.clone())).unwrap()).unwrap();
            writeln!(&mut params, "store [r0] r1").unwrap();
        }
        for l in params.lines() {
            writeln!(&mut main_func, "    {}", l).unwrap();
        }

        let mut body = String::new();
        for stmt in &func.body {
            compile_stmt(&mut body, stmt, &func.name, &variables_table).unwrap();
        }

        // if we forgot to ret at this point
        writeln!(&mut body, "; Oh no we got out of the scope !").unwrap();
        match func.return_ty {
            Type { base: BaseType::Void, ptr: _ptr } => {
                writeln!(&mut body, "; Eh its fine we are in a void func").unwrap();
                writeln!(&mut body, "; we push a NULL").unwrap();
                writeln!(&mut body, "pop r0 ; we push a NULL").unwrap();
                writeln!(&mut body, "push 0").unwrap();
                writeln!(&mut body, "push r0").unwrap();
                writeln!(&mut body, "ret").unwrap();
            },
            _ => {
                writeln!(&mut body, "; Oh no we are in trouble").unwrap();
                // showing red for now
                writeln!(&mut body, "jump red").unwrap();
            }
        }

        for l in body.lines() {
            writeln!(&mut main_func, "    {}", l).unwrap();
        }

        writeln!(&mut main_func, "; end of func {}\n", &func.name).unwrap();
    }

    let assembly_output = format!(r#"; init of stack
let r0 0x00fffffc
copy sp r0
; entrypoint
jump _start
_start:
    ; no args possible for now
    call main
    ; handling of exit codes
    pop r0
    skip 1 ifne r0 0
    jump green ; true
    jump red ; false

{}

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