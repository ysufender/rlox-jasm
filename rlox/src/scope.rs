use rustc_hash::FxHashMap;

use crate::{lox::LoxError, lox_value::LoxValue, symbol::SymbolTable};

pub struct Scope<'a> {
    parent: Option<&'a Scope<'a>>,
    scope_variables: rustc_hash::FxHashMap<usize, (usize, usize, LoxValue)>,
    scope_signatures: rustc_hash::FxHashMap<usize, Vec<usize>>,
    pos: usize,
    generation: usize
}

impl<'a> Scope<'a> {
    pub fn new(parent: Option<&'a Scope<'a>>, gen: Option<usize>) -> Scope<'a> {
        Scope {
            parent: parent,
            scope_variables: FxHashMap::default(),
            scope_signatures: FxHashMap::default(),
            pos: 0,
            generation: if let Some(p) = parent { p.generation+1 }
                        else if let Some(g) = gen { g }
                        else { 2 }
        }
    }
    
    pub fn gen(&self) -> usize { self.generation }

    pub fn has_parent(&self) -> bool { matches!(self.parent, Some(_)) }

    pub fn add_signature(&mut self, name: usize, signature: &Vec<usize>, symbol_table: &SymbolTable) -> Result<(), LoxError> {
        if self.scope_signatures.contains_key(&name) {
            Err(LoxError::CompilationError(format!("Given function '{}' already exists in this scope.", symbol_table.get_symbols().get(name).unwrap())))
        } else {
            self.scope_signatures.insert(name, signature.clone());
            Ok(())
        }
    }

    pub fn add_var(&mut self, name: usize, size: usize, symbol_table: &SymbolTable, value: LoxValue) -> Result<(), LoxError> {
        if self.scope_variables.contains_key(&name) {
            Err(LoxError::CompilationError(format!("Given variable '{}' already exists in this scope.", symbol_table.get_symbols().get(name).unwrap())))
        }
        else {
            self.scope_variables.insert(name, (self.pos, size, value));
            self.pos += size;
            Ok(())
        }
    }

    pub fn get_var(&self, name: usize, symbol_table: &SymbolTable) -> Result<(usize, usize, LoxValue), LoxError> {
        if let Some(&(pos, size, ref val)) = self.scope_variables.get(&name) {
            Ok((pos, size, val.clone()))
        }
        else if let Some(parent) = self.parent {
            parent.get_var(name, symbol_table)
        }
        else {
            Err(LoxError::CompilationError(format!("Couldn't find variable '{}' in the current scope", symbol_table.get_symbols().get(name).unwrap())))
        }
    }
}
