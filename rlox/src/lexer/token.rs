use std::fmt;
use std::hash::Hash;

use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use crate::lox_value::LoxValue;
use crate::symbol::{Symbol, SymbolTable};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Colon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Arrow,

    // Literals & Types
    Identifier,
    String,
    Number,
    Bool,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Void,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Eof,
}

impl From<TokenType> for Literal {
    fn from(value: TokenType) -> Self {
        match value {
            TokenType::String => Self::Str("".into()),
            TokenType::Number => Self::Num(0f64.into()),
            TokenType::Bool => Self::False,
            _ => unreachable!()
        }
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone)]
pub struct Hf64(pub Decimal);

impl From<f64> for Hf64 {
    fn from(value: f64) -> Self {
        Hf64(Decimal::from_f64(value).unwrap())
    }
}

impl From<Hf64> for f64 {
    fn from(val: Hf64) -> Self {
        val.0.to_f64().unwrap()
    }
}

impl From<&Hf64> for f64 {
    fn from(val: &Hf64) -> Self {
        val.0.to_f64().unwrap()
    }
}

impl fmt::Display for Hf64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Literal {
    Str(String),
    Num(Hf64),
    True,
    False,
    Void,
}

pub struct ErrorToken {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Literal,
    pub line: usize,
}

impl ErrorToken {
    pub fn new(token: &Token, symbol_table: &SymbolTable) -> Self {
        Self {
            token_type: token.token_type.clone(),
            lexeme: symbol_table.resolve(token.lexeme).to_string(),
            literal: token.literal.clone(),
            line: token.line,
        }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: Symbol,
    pub literal: Literal,
    pub line: usize,
}

impl From<Token> for Literal {
    fn from(value: Token) -> Self {
        match value.token_type {
            TokenType::String => Self::Str("".into()),
            TokenType::Number => Self::Num(0f64.into()),
            TokenType::Bool => Self::False,
            TokenType::Void => Self::Void,
            _ => unreachable!()
        }
    }
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: Symbol, literal: Literal, line: usize) -> Self {
        Token {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let literal = match &self.literal {
            Literal::Str(str) => str.clone(),
            Literal::Num(num) => num.to_string(),
            Literal::Void => "".to_string(),
            Literal::True => "true".to_string(),
            Literal::False => "false".to_string(),
        };
        write!(
            f,
            "line {}: {:?} {} {}",
            self.line, self.token_type, self.lexeme, literal
        )
    }
}
