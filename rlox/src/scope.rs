use std::{cell::RefCell, rc::Rc};

use rust_decimal::prelude::Signed;
use rustc_hash::FxHashMap;

use crate::{lexer::token::Token, lox::LoxError, lox_value::LoxValue, symbol::{Symbol, SymbolTable}};

pub type ScopeRef = Rc<RefCell<Scope>>;

pub struct Scope {
    parent: Option<ScopeRef>,
    scope_variables: rustc_hash::FxHashMap<usize, (usize, usize, LoxValue)>,
    scope_signatures: rustc_hash::FxHashMap<usize, (usize, LoxValue, Rc<RefCell<Vec<LoxValue>>>)>,
    pos: usize,
    generation: usize,
    scope_name: Option<Symbol> 
}

impl Scope {
    pub fn new(parent: Option<ScopeRef>, gen: Option<usize>, name: Option<Symbol>) -> Scope {
        let genereation = if let Some(g) = gen { g }
                          else if let Some(ref p) = parent { p.borrow().gen()+1 }
                          else { 1 };
        Scope {
            parent: parent,
            scope_variables: FxHashMap::default(),
            scope_signatures: FxHashMap::default(),
            pos: 0,
            generation: genereation,
            scope_name: name
        }
    }

    pub fn gen(&self) -> usize { self.generation }
    pub fn id(&self) -> &Option<Symbol> { &self.scope_name }
    pub fn has_parent(&self) -> bool { matches!(self.parent, Some(_)) }
    pub fn pos(&self) -> usize { self.pos }

    pub fn is_void(&self) -> bool { 
        if let Some(Symbol(sym)) = self.scope_name {
            let &(_, returns, _) = &self.scope_signatures.get(&sym).unwrap();
            returns.size() == 0
        } else { true }
    }

    pub fn add_signature(&mut self, name: Symbol, signature: &(usize, LoxValue, Rc<RefCell<Vec<LoxValue>>>), symbol_table: &SymbolTable) -> Result<(), LoxError> {
        if self.scope_signatures.contains_key(&name.0) {
            Err(LoxError::CompilationError(format!("Given function '{}' already exists in this scope.", symbol_table.resolve(name))))
        } else {
            self.scope_signatures.insert(name.0, signature.clone());
            Ok(())
        }
    }

    pub fn get_signature(&self, name: Symbol, symbol_table: &SymbolTable) -> Result<(usize, LoxValue, Rc<RefCell<Vec<LoxValue>>>), LoxError> {
        if self.scope_signatures.contains_key(&name.0) {
            Ok(self.scope_signatures.get(&name.0).unwrap().clone())
        } else if let Some(parent) = &self.parent {
            let p = parent.borrow();
            p.get_signature(name, symbol_table)
        } else {
            Err(LoxError::CompilationError(format!("Given function '{}' already exists in this scope.", symbol_table.resolve(name))))
        }
    }

    pub fn has_signature(&self, name: Symbol) -> bool {
        if self.scope_signatures.contains_key(&name.0) { true }
        else if let Some(parent) = &self.parent { parent.borrow().has_signature(name) }
        else { false }
    }

    pub fn add_var(&mut self, name: Symbol, size: usize, symbol_table: &SymbolTable, value: LoxValue) -> Result<(), LoxError> {
        if self.scope_variables.contains_key(&name.0) {
            Err(LoxError::CompilationError(format!("Given variable '{}' already exists in this scope.", symbol_table.resolve(name))))
        }
        else {
            self.scope_variables.insert(name.0, (self.pos, size, value));
            self.pos += size;
            Ok(())
        }
    }

    pub fn get_var(&self, name: Symbol, symbol_table: &SymbolTable) -> Result<(usize, usize, LoxValue), LoxError> {
        if let Some(&(pos, size, ref val)) = self.scope_variables.get(&name.0) {
            Ok((pos, size, val.clone()))
        }
        else if let Some(parent) = &self.parent {
            let p = parent.borrow();
            p.get_var(name, symbol_table)
        }
        else {
            Err(LoxError::CompilationError(format!("Couldn't find variable '{}' in the current scope", symbol_table.resolve(name))))
        }
    }

    pub fn has_var(&self, name: Symbol) -> bool {
        if self.scope_variables.contains_key(&name.0) { true }
        else if let Some(parent) = &self.parent { parent.borrow().has_var(name) }
        else { false }
    }
}
