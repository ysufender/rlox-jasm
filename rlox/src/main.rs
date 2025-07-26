use std::env::args;
use std::process::exit;

use rlox_jasm::lox;

fn main() {
   let args: Vec<String> = args().collect();
    let args_str: Vec<&str> = args.iter().map(String::as_str).collect();

    match args_str.as_slice() {
        ["run", files @ ..] if !files.is_empty() => {
        }
        ["build", files @ ..] if !files.is_empty() => {
        }
        ["jasm", files @ ..] if !files.is_empty() => {
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
