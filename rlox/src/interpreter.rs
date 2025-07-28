use std::cell::RefCell;
use std::fs::File;
use std::io::Write;
use std::mem::discriminant;
use rustc_hash::FxHashMap;
use std::rc::Rc;

use crate::environment::{Environment, EnvironmentError};
use crate::expr::{Expr, ExprIdx, ExprPool};
use crate::lexer::token::Literal;
use crate::lexer::token::Token;
use crate::lexer::token::TokenType;
use crate::lox::{LoxError};
use crate::scope::{Scope, ScopeRef};
use crate::stmt::Stmt;
use crate::lox_value::{LoxValue, LoxValueError};
use crate::symbol::{SymbolTable};

macro_rules! generate {
   ($out:expr, $tab_count:expr, $($line:expr),* $(,)?) => {{
        let tabs = "\t".repeat($tab_count);
        $(
            if (!$line.is_empty()) {
                write!($out, "\n{}{}", tabs, $line)?;
            }
        )* 
        //write!($out, "\n")?;
        Ok(LoxValue::Void) as Result<LoxValue, LoxError>
    }};
}

macro_rules! f {
    ($($tt:tt)*) => {
        format!($($tt)*)
    };
}

#[derive(Debug)]
pub enum RuntimeError {
    IncorrectOperand(Token, LoxValueError),
    DivideByZero(Token, LoxValueError),
    InterpreterPanic(Token, String),
    InstanceError(Token, String),
    UndefinedVariable(Token, EnvironmentError),
    AssignVariableError(Token, EnvironmentError),
    InputError(String),
    CustomError(String),
    Return(LoxValue),
}

impl RuntimeError {
    pub fn get_info(self) -> (Token, String) {
        match self {
            RuntimeError::IncorrectOperand(token, lox_value_err)
            | RuntimeError::DivideByZero(token, lox_value_err) => {
                (token, lox_value_err.get_string())
            }
            RuntimeError::UndefinedVariable(token, environment_err)
            | RuntimeError::AssignVariableError(token, environment_err) => {
                (token, environment_err.get_string())
            }
            RuntimeError::InterpreterPanic(token, err_str)
            | RuntimeError::InstanceError(token, err_str) => (token, err_str),
            _ => panic!("Should not reach here!"),
        }
    }
}

#[derive(Debug)]
pub struct Interpreter<'a> {
    //globals: Rc<RefCell<Environment>>,
    //environment: Rc<RefCell<Environment>>,
    //locals: FxHashMap<ExprIdx, usize>,
    expr_pool: &'a ExprPool,
    pub symbol_table: &'a mut SymbolTable,
    counter: usize
}

impl<'a> Interpreter<'a> {
    pub fn new(expr_pool: &'a ExprPool, symbol_table: &'a mut SymbolTable/*, _: FxHashMap<ExprIdx, usize>*/) -> Self {
        // Create a new global environment
        //let globals = Environment::new();
        //define_globals(&globals, symbol_table);

        // Initially, the environment is the global environment
        //let environment = Rc::clone(&globals);

        Self {
            //globals,
            //environment,
            //locals,
            expr_pool,
            symbol_table,
            counter: 0
        }
    }

    pub fn gen_label(&mut self, pre: &str) -> String {
        let c = self.counter;
        self.counter += 1;
        f!("{}{}", pre, c)
    }

    pub fn get_globals(&self) -> Rc<RefCell<Environment>> {
        //Rc::clone(&self.globals)
        unreachable!()
    }

    pub fn gen_il(&mut self, statements: &[Stmt], out: &mut File, cur_scope: Option<ScopeRef>) -> Result<LoxValue, LoxError> {
        let scope = match cur_scope {
            None => Rc::new(RefCell::new(Scope::new(None, None, None))),
            Some(s) => s
        };
        for statement in statements {
            if scope.clone().borrow().gen() <= 1 && !matches!(statement, Stmt::Function{..}) {
                return Err(LoxError::CompilationError("Top level statements are not allowed.".into()));
            }

            match statement {
                Stmt::Expression { expression } => {
                    let _ = self.handle_expression(self.expr_pool.get_expr(*expression), out, scope.clone())?;
                }
                Stmt::Print { expression } => {
                    let expr = self.expr_pool.get_expr(*expression);
                    let val = self.handle_expression(expr, out, scope.clone())?;
                    match val {
                        LoxValue::String(_) | LoxValue::Variable(_, _) => {
                            generate!(out, scope.borrow().gen(), 
                                "#Print#",
                                "mov &ebx",
                                "mov &ebx &eax",
                                "rda %i",
                                "mov &ecx",
                                "pop %i",
                                "mov &sp &ebx",
                                "inc %i &ecx 4",
                                "mcp %h %s",
                                "add %i &ecx &sp",
                                "mov 1 &dl",
                                "or &dl &flg",
                                "mov &ecx &bl",
                                "cal 0x0",
                                "dcr %b &flg 1",
                                "sub %i &sp &ecx",
                                "mov &ecx &sp"
                            )?;
                        }
                        _ => return Err(LoxError::CompilationError("Expected string".into()))
                    }
                },
                Stmt::Var { name, initializer }  => 
                    if name.token_type == TokenType::Identifier {
                        let expr = self.expr_pool.get_expr(initializer.unwrap());
                        generate!(out, scope.borrow().gen(), format!("#variable {}#", self.symbol_table.resolve(name.lexeme)))?;
                        let val = self.handle_expression(expr, out, scope.clone())?;
                        scope.borrow_mut().add_var(name.lexeme, val.size(), self.symbol_table, val)?;
                    } else { return Err(LoxError::CompilationError("Expected identifier".into())); },
                Stmt::Block { statements } => { 
                    generate!(out, scope.borrow().gen(), "#block#")?;
                    let b_scope = Rc::new(RefCell::new(Scope::new(Some(scope.clone()), None, None)));
                    let _ = self.gen_il(&statements, out, Some(b_scope.clone()))?;
                    generate!(out, scope.borrow().gen()+1, f!("dcr %i &sp {}", b_scope.borrow().pos()))?;
                }
                Stmt::If { condition: _, then_branch: _, else_branch : _} => return Ok(LoxValue::Void),
                Stmt::While { condition: _, body : _} => return Ok(LoxValue::Void),
                Stmt::Function { name, params, return_type, body } => {
                    let var = self.symbol_table.resolve(name.lexeme);
                    generate!(out, scope.borrow().gen(), 
                        "#function definition#",
                        f!("#{}({}) -> {}#", var, params.len(), return_type),
                        f!("{}:", var)
                    )?;
                    let mut fn_scope = Scope::new(Some(scope.clone()), None, Some(name.lexeme));
                    let mut params_returns: Vec<LoxValue> = Vec::new();
                    if params.len() != 0 { 
                        for Token { token_type: _, lexeme, literal, line: _ } in params {
                            let val: LoxValue = From::<&Literal>::from(literal);
                            fn_scope.add_var(
                                *lexeme, 
                                <LoxValue as From::<&Literal>>::from(literal).size(),
                                self.symbol_table,
                                val.clone()
                            )?;
                            params_returns.push(val);
                        }
                    }
                    let signature = (params.len(), return_type.clone(), Rc::new(RefCell::new(params_returns)));
                    scope.borrow_mut().add_signature(name.lexeme, &signature, self.symbol_table)?;
                    fn_scope.add_signature(name.lexeme, &signature, self.symbol_table)?;
                    let _ = self.gen_il(&body, out, Some(Rc::new(RefCell::new(fn_scope))))?;
                },
                Stmt::Return { keyword: _, value } => {
                    let borrowed_scope = scope.borrow();
                    let ret_type: LoxValue = if let Some(exprid) = value {
                        generate!(out, borrowed_scope.gen(), "#return eval#")?;
                        let expr = self.expr_pool.get_expr(*exprid);    
                        let val = self.handle_expression(expr, out, scope.clone());
                        if let Err(e) = val {
                            return Err(e)
                        } else { val? }
                    } else { LoxValue::Void };
                    let fn_ret_type = &borrowed_scope.get_signature(borrowed_scope.id().unwrap(), self.symbol_table)?.1;
                    if discriminant(fn_ret_type) != discriminant(&ret_type) {
                        return Err(LoxError::CompilationError("Return type doesn't match with function signature.".into()));
                    }
                    generate!(out, scope.borrow().gen(),
                        "#return#",
                        f!("mov {} &bl", ret_type.size()),
                        "ret"
                    )?;
                    return Ok(LoxValue::Void);
                },
                Stmt::Class { name: _, superclass: _, methods : _} => unreachable!("Classes are not yesupported."),
                _ => unreachable!("This should've been unreachable")
            }
        }

        //if !scope.has_parent() && scope.gen() > 2 {
        if !scope.borrow().is_void() {
            Err(LoxError::CompilationError("Expected return at the end of function body.".into()))
        } else { Ok(LoxValue::Void) }
    }

    fn handle_expression(&mut self, expr: &Expr, out: &mut File, scope: ScopeRef) -> Result<LoxValue, LoxError> {
        match expr {
            Expr::Binary { left, operator, right } => {
                let lhs = self.handle_expression(self.expr_pool.get_expr(*left), out, scope.clone())?;
                let rhs = self.handle_expression(self.expr_pool.get_expr(*right), out, scope.clone())?;
                if lhs.r#type() != rhs.r#type() { return Err(LoxError::CompilationError("Operand type missmatch.".into())) }
                match operator.token_type {
                    TokenType::Plus => {generate!(out, scope.borrow().gen(), "add %f")?; Ok(lhs)},
                    _ => todo!("Not Yet")
                }
            },
            Expr::Grouping { expression } => self.handle_expression(self.expr_pool.get_expr(*expression), out, scope.clone()),
            Expr::Literal { value } => match value {
                Literal::Str(val) =>
                    if !val.is_ascii() { Err(LoxError::CompilationError("Only ASCII strings are accepted.".to_string())) } 
                    else { generate!(out, scope.borrow().gen(),
                                "#str literal#",
                                f!("raw {} \"{}\" ;", val.len(), val),
                                f!("mov {} &ecx", val.len()+4), 
                                "alc", 
                                "mov &ecx &eax",
                                "sub %i &sp &eax",
                                "mcp %s %h",
                                f!("dcr %i &sp {}", val.len()+4), 
                                "rda &ebx"
                            )?; Ok(LoxValue::String(val.to_string())) },
                Literal::Num(val) => { generate!(out, scope.borrow().gen(), "#num literal#", f!("stc %f {}", val))?; Ok(LoxValue::Number(val.into())) },
                Literal::True => { generate!(out, scope.borrow().gen(), "#bool literal#", "stc %b 1")?; Ok(LoxValue::Boolean(true)) },
                Literal::False => { generate!(out, scope.borrow().gen(), "#bool literal#", "stc %b 0")?; Ok(LoxValue::Boolean(false)) }
                _ => unreachable!("Void values are not implemented yet.")
            },
            Expr::Unary { operator, right } => Ok(LoxValue::Void),
            Expr::Variable { name } => match name.token_type {
                TokenType::Identifier => {
                    let var_name = self.symbol_table.resolve(name.lexeme);
                    if scope.borrow().has_var(name.lexeme) {
                        let (pos, size, var_t) = scope.borrow().get_var(name.lexeme, self.symbol_table)?;
                        generate!(out, scope.borrow().gen(), 
                            f!("#var ref {}#", var_name),
                            "mov &bp &ebx",
                            if pos > 0 { f!("inc %i &ebx {}", pos) } else { "".into() },
                            if size == 1 { "rda %b" } else { "rda %i" },
                        )?;
                        Ok(var_t)
                    } else if scope.borrow().has_signature(name.lexeme) {
                        generate!(out, scope.borrow().gen(), f!("#fn ref {}#", var_name))?;
                        Ok(LoxValue::Fn(name.lexeme))
                    } else {
                        Err(LoxError::CompilationError(format!("Couldn't find '{}' in the current scope", var_name)))
                    }
                },
                _ => Err(LoxError::CompilationError("Expected identifier".into()))
            },
            Expr::Assign { name: Token { token_type: TokenType::Identifier, lexeme, literal:_, line:_ }, value } => {
                let expr = self.expr_pool.get_expr(*value); 
                let (pos, _, var) = scope.borrow().get_var(*lexeme, self.symbol_table)?;
                generate!(out, scope.borrow().gen(), f!("#assignment {}#", pos))?;
                let val = self.handle_expression(expr, out, scope.clone())?;
                if var.r#type() != val.r#type() {
                    return Err(LoxError::CompilationError("Type missmatch.".into()));
                }
                match val {
                    LoxValue::String(_) => {
                        generate!(out, scope.borrow().gen(),
                            "#string assignment#",
                            "mov &bp &ebx",
                            if pos > 0 { f!("inc %i &ebx {}", pos) } else { "".into() },
                            "rda %i",
                            "mov &ebx",
                            "pop %i",
                            "rda %i",
                            "mov &ecx",
                            "pop %i",
                            "inc %i &ecx 4",
                            "del",
                            "mov &bp &ebx",
                            if pos > 0 { f!("inc %i &ebx {}", pos) } else { "".into() },
                            "ldc %i",
                            "pop %i"
                        )?;
                        Ok(LoxValue::Void)
                    },
                    LoxValue::Void => Err(LoxError::CompilationError("Void assignation is not permitted.".into())),
                    LoxValue::Callable(_) => Err(LoxError::CompilationError("Can't assign functions to things.".into())),
                    _ => {
                        generate!(out, scope.borrow().gen(), 
                            "#assignment#",
                            "mov &bp &ebx",
                            if pos > 0 { f!("inc %i &ebx {}", pos) } else { "".into() },
                            if val.size() == 1 { "ldc %b" } else { "ldc %i" },
                            if val.size() == 1 { "pop %b" } else { "pop %i" },
                        )?;
                        Ok(LoxValue::Void)
                    }
                }
            },
            Expr::Logical { left: _, operator: _, right : _} => Ok(LoxValue::Void),
            Expr::Call { callee, paren: _, arguments } => {
                let calid = self.expr_pool.get_expr(*callee);
                let val = self.handle_expression(calid, out, scope.clone())?;
                let mut size = 0;
                if let LoxValue::Fn(name) = val {
                    generate!(out, scope.borrow().gen(), "#function call#")?;
                    let fn_sign = scope.borrow().get_signature(name, self.symbol_table)?;
                    if fn_sign.0 != arguments.len() {
                        Err(LoxError::CompilationError("Missmatching parameter count.".into()))
                    } else {
                        if fn_sign.0 != 0 { generate!(out, scope.borrow().gen(), "#parameters#")?; }
                        for (idx, exprid) in arguments.into_iter().enumerate() {
                            let param = self.handle_expression(self.expr_pool.get_expr(*exprid), out, scope.clone())?;
                            if discriminant(&param) != discriminant(fn_sign.2.borrow().get(idx).unwrap()){
                                return Err(LoxError::CompilationError("Missmatching parameter types.".into()))
                            }
                            if size + param.size() > 255 {
                                return Err(LoxError::CompilationError("Parameter size is too big (max 255)".into()))
                            }
                            size += param.size();
                        }
                        generate!(out, scope.borrow().gen(),
                            "#call#",
                            f!("mov {} &bl", size),
                            f!("cal {}", self.symbol_table.resolve(name))
                        )?;
                        Ok(fn_sign.1)
                    }
                } else {
                    Err(LoxError::CompilationError("Attempt to call non-function value.".into()))
                }
            },
            Expr::Get { object: _, name : _} => todo!("Classes are not yet supported."),
            Expr::Set { object: _, name: _, value : _} => todo!("Classes are not yet supported."),
            Expr::This { keyword : _} => todo!("Classes are not yet supported."),
            Expr::Super { keyword: _, method : _} => todo!("Classes are not yet supported."),
            _ => todo!()
        }
    }
}
