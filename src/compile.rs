use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

use crate::{ast::*, parser};

pub fn bisare(main_ac: PathBuf) -> std::io::Result<()> {
    let src = fs::read_to_string(&main_ac)?;
    let main_asm = main_ac.with_extension("asm");
    let main_bin = main_ac.with_extension("bin");

    let ast = parser::parse_program(&src)
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    println!("{:#?}", ast);

    let output_code = match ast {
        Program {
            func: Function { body, .. },
        } => match body.as_slice() {
            [Stmt::Return(0)] => "green",
            [Stmt::Return(i)] => "red",
            _ => return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "program must contain exactly one `return` statement",
            )),
        },
    };

    let asm_output = format!(r#"main:
    jump {}

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
"#, output_code);

    fs::write(&main_asm, asm_output)?;

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