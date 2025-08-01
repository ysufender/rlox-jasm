use std::rc::Rc;

use crate::expr::ExprIdx;
use crate::lexer::token::Token;
use crate::lox_value::LoxValue;

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Expression {
        expression: ExprIdx,
    },
    Print {
        expression: ExprIdx,
    },
    Var {
        name: Token,
        initializer: Option<ExprIdx>,
    },
    Block {
        statements: Vec<Stmt>,
    },
    If {
        condition: ExprIdx,
        then_branch: Rc<Stmt>,
        else_branch: Option<Rc<Stmt>>,
    },
    While {
        condition: ExprIdx,
        body: Rc<Stmt>,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        return_type: LoxValue,
        body: Vec<Stmt>,
    },
    Return {
        keyword: Token,
        value: Option<ExprIdx>,
    },
    Class {
        name: Token,
        superclass: Option<ExprIdx>,
        methods: Vec<Stmt>,
    },
}

impl From<Stmt> for Option<ExprIdx> {
    fn from(val: Stmt) -> Self {
        match val {
            Stmt::Expression { expression } | Stmt::Print { expression } => Some(expression),
            Stmt::Var {
                name: _,
                initializer,
            } => initializer,
            _ => panic!("Should not be reached!"),
        }
    }
}
