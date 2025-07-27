use rust_decimal::prelude::Signed;
use rustc_hash::FxHashMap;

use crate::{lox::LoxError, lox_value::LoxValue, symbol::{Symbol, SymbolTable}};

pub struct Scope<'a> {
    parent: Option<&'a Scope<'a>>,
    scope_variables: rustc_hash::FxHashMap<usize, (usize, usize, LoxValue)>,
    scope_signatures: rustc_hash::FxHashMap<usize, (usize, usize, &'a Vec<LoxValue>)>,
    pos: usize,
    generation: usize,
    scope_name: Option<Symbol> 
}

impl<'a> Scope<'a> {
    pub fn new(parent: Option<&'a Scope<'a>>, gen: Option<usize>, name: Option<&Symbol>) -> Scope<'a> {
        Scope {
            parent: parent,
            scope_variables: FxHashMap::default(),
            scope_signatures: FxHashMap::default(),
            pos: 0,
            generation: if let Some(p) = parent { p.generation+1 }
                        else if let Some(g) = gen { g }
                        else { 2 },
            scope_name: name.copied()
        }
    }
    
    pub fn gen(&self) -> usize { self.generation }
    pub fn id(&self) -> &Option<Symbol> { &self.scope_name }
    pub fn has_parent(&self) -> bool { matches!(self.parent, Some(_)) }
    pub fn pos(&self) -> usize { self.pos }

    pub fn is_void(&self) -> bool { 
        if let Some(Symbol(sym)) = self.scope_name {
            let &(_, returns, _) = self.scope_signatures.get(&sym).unwrap();
            returns == 0
        } else { true }
    }

    pub fn add_signature(&mut self, name: Symbol, signature: &(usize, usize, &'a Vec<LoxValue>), symbol_table: &SymbolTable) -> Result<(), LoxError> {
        if self.scope_signatures.contains_key(&name.0) {
            Err(LoxError::CompilationError(format!("Given function '{}' already exists in this scope.", symbol_table.resolve(name))))
        } else {
            self.scope_signatures.insert(name.0, signature.clone());
            Ok(())
        }
    }

    pub fn get_signature(&self, name: Symbol, symbol_table: &SymbolTable) -> Result<&(usize, usize, &'a Vec<LoxValue>), LoxError> {
        if self.scope_signatures.contains_key(&name.0) {
            Ok(self.scope_signatures.get(&name.0).unwrap())
        } else {
            Err(LoxError::CompilationError(format!("Given function '{}' already exists in this scope.", symbol_table.resolve(name))))
        }
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
        else if let Some(parent) = self.parent {
            parent.get_var(name, symbol_table)
        }
        else {
            Err(LoxError::CompilationError(format!("Couldn't find variable '{}' in the current scope", symbol_table.resolve(name))))
        }
    }
}
