use std::env::{self, args};
use std::process::exit;

use rlox_jasm::lox;

macro_rules! log_if_err {
    ($expr:expr) => {
        if let Err(err) = $expr {
            eprintln!("Error: {}", err);
        }
    };
}

fn main() {
    let args: Vec<String> = args().collect();
    let args_str: Vec<&str> = args[1..].iter().map(String::as_str).collect();
    env::set_var("RUST_BACKTRACE", "1");

    match args_str.as_slice() {
        ["run", "interpret", files @ ..] => {
            log_if_err!(lox::interpret_files(files));
        },
        ["run", files @ ..] if !files.is_empty() => {
            log_if_err!(lox::run_files(files));
        }
        ["build", files @ ..] if !files.is_empty() => {
            log_if_err!(lox::build_files(files));
        }
        ["jasm", files @ ..] if !files.is_empty() => {
            log_if_err!(lox::jasm_files(files));
        }
        ["help", ..] | _ => print_usage()
    }
}

fn print_usage() {
    println!(
"
rlox-jasm --- A JASM IL and Bytecode generating lox compiler written in Rust.

Usaage:
    rlox-jasm run   <..files..>     : Convert all given source files to JASM Bytecode and execute.
    rlox-jasm build <..files..>     : Convert all given source files to JASM Bytecode, but don't execute.
    rlox-jasm jasm  <..files..>     : Convert all given source files to JASM IL.
    rlox-jasm help                  : Print this message.
");
    exit(64);
}
