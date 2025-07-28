use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::env;
use std::process::Command;
use std::io;
use std::io::{stdin, stdout, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};


use crate::interpreter::{Interpreter, RuntimeError};
use crate::lexer::scanner;
use crate::lexer::token::{ErrorToken, TokenType};
use crate::parser::Parser;
use crate::symbol::SymbolTable;

#[derive(Debug)]
pub enum LoxError {
    IOError(io::Error),
    Error(String),
    RuntimeError(String),
    CompilationError(String)
}

impl fmt::Display for LoxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoxError::IOError(err) => write!(f, "IO Error: {}", err),
            LoxError::Error(msg) => write!(f, "Error: {}", msg),
            LoxError::RuntimeError(msg) => write!(f, "Runtime Error: {}", msg),
            LoxError::CompilationError(msg) => write!(f, "Error while compiling lox file: {}", msg)
        }
    }
}

impl Error for LoxError {}

impl From<io::Error> for LoxError {
    fn from(err: io::Error) -> LoxError {
        LoxError::IOError(err)
    }
}

static mut HAD_ERROR: bool = false;
static mut HAD_RUNTIME_ERROR: bool = false;

fn check_errors() -> Result<(), LoxError> {
    unsafe {
        if HAD_ERROR {
            return Err(LoxError::Error("Compilation error".to_string()));
        }
        if HAD_RUNTIME_ERROR {
            return Err(LoxError::RuntimeError("Runtime error".to_string()));
        }
    }
    Ok(())
}

pub fn run_files(files: &[&str]) -> Result<(), LoxError> {
    let byte_files = build_files(files)?;

    // invoke CSR to run byte_files
    let status = Command::new(env::current_exe()?.parent().unwrap().join("csr"))
        .arg("-e").args(byte_files)
        .status()?;
    
    if !status.success() {
        Err(LoxError::CompilationError(format!("Failed to invoke csr [{}]", status.to_string())))
    }
    else {
        Ok(())
    }
}

pub fn build_files(files: &[&str]) -> Result<Vec<String>, LoxError> {
    let mut res: Vec<String> = Vec::new();
    let il_files = jasm_files(files)?;
    for il_file in il_files {
        let source_path = Path::new(&il_file); 
        let mut dest_path: PathBuf = source_path.parent().unwrap().to_path_buf();
        dest_path.push(format!(
            "{}.jef",
            source_path.file_stem().unwrap().to_str().unwrap()
        ));
        
        let status = Command::new(env::current_exe()?.parent().unwrap().join("jasm"))
            .args(["-s", "-I", &il_file, "-o", dest_path.as_path().to_str().unwrap()])
            .status()?;

        if !status.success() {
            return Err(LoxError::CompilationError(format!("Failed to invoke jasm [{}]", status.to_string())));
        }

        res.push(dest_path.into_os_string().into_string().unwrap());
    }

    Ok(res)
}

pub fn jasm_files(files: &[&str]) -> Result<Vec<String>, LoxError> {
    let mut res: Vec<String> = Vec::new();

    for source in files {
        let source_path = Path::new(source); 
        let mut dest_path: PathBuf = source_path.parent().unwrap().to_path_buf();
        dest_path.push(format!(
            "{}.jasm",
            source_path.file_stem().unwrap().to_str().unwrap()
        ));

        let mut output = File::create(&dest_path)?;
        write!(output,
"This file has been generated automatically by rlox-jasm.
rlox, Rust implementation of lox from Crafting Interpreters by Emirhan TALA.
rlox-jasm, JASM IL and Bytecode generation for rlox by Yusuf Ender Osmanoğlu.

.prep
    org __jasm_IL_entry_main__
    sts #1032# 32
    sth #1024# 32
.body
    __jasm_IL_entry_main__:
        cal main
        jmp __jasm_IL_end__
")?;

        // turn AST into bytecode
        let src = std::fs::read_to_string(source)?;
        let result = compile(&src, &mut output);

        if result.is_err() {
            return Err(result.unwrap_err());
        }

        write!(output, "\n__jasm_IL_end__:\n.end\n\nEnd of generated IL.")?;
        res.push(dest_path.into_os_string().into_string().unwrap());
    }

    Ok(res)
}

pub fn run_prompt() -> Result<(), LoxError> {
    let stdin = stdin();
    let input = stdin.lock();
    let mut reader = BufReader::new(input);

    loop {
        print!("> ");
        stdout().flush()?; // Ensure the prompt is displayed immediately

        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line)?;

        if bytes_read == 0 {
            break; // EOF reached
        }

        //run(line.trim())?;

        unsafe {
            HAD_ERROR = false;
        }
    }

    Ok(())
}

pub fn compile(source: &str, out: &mut File) -> Result<(), LoxError> {
    let mut symbol_table = SymbolTable::new(); // For the lexer.
    let lexer_tokens = {
        let mut lexer = scanner::Scanner::new(source, &mut symbol_table);
        lexer.scan_tokens();

        lexer.tokens
    };
    let parser = Parser::new(&symbol_table, lexer_tokens);
    let (statements, expr_pool) = parser
        .parse()
        .map_err(|_| LoxError::Error("Error during parsing".into()))?;
    check_errors()?;

    //let locals = Resolver::new(&expr_pool, &mut symbol_table).resolve_lox(&statements);

    let mut interpreter = Interpreter::new(&expr_pool, &mut symbol_table);
    interpreter.gen_il(&statements, out, None)?;
    Ok(())
}

pub fn error(token: &ErrorToken, message: &str) {
    if token.token_type == TokenType::Eof {
        report(token.line, " at end", message);
    } else {
        report(token.line, &format!("at '{}'", token.lexeme), message);
    }
}

pub fn report(line_num: usize, line: &str, message: &str) {
    eprintln!("[line {line_num}] {:?}: {message}", line);

    unsafe {
        HAD_ERROR = true;
    }
}

pub fn runtime_error(error: RuntimeError) {
    let (token, err_msg) = error.get_info();
    eprintln!("{}\n[line {}]", err_msg, token.line);

    unsafe {
        HAD_RUNTIME_ERROR = true;
    }
}
