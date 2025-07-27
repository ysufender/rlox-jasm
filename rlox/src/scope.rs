use rustc_hash::FxHashMap;

use crate::lox::LoxError;

pub struct Scope<'a> {
    parent: Option<&'a Scope<'a>>,
    scope_variables: rustc_hash::FxHashMap<String, (usize, usize)>,
    pos: usize,
    generation: usize
}

impl<'a> Scope<'a> {
    pub fn new(parent: Option<&'a Scope<'a>>) -> Scope<'a> {
        Scope {
            parent: parent,
            scope_variables: FxHashMap::default(),
            pos: 0,
            generation: if let Some(p) = parent { p.generation+1 } else { 2 }
        }
    }
    
    pub fn gen(&self) -> usize { self.generation }

    pub fn add_var(&mut self, name: &String, size: usize) -> Result<(), LoxError> {
        if self.scope_variables.contains_key(name) {
            Err(LoxError::CompilationError(format!("Given variable {} already exists in this scope.", name)))
        }
        else {
            self.scope_variables.insert(name.to_string(), (self.pos, size));
            self.pos += size;
            Ok(())
        }
    }

    pub fn get_var(&self, name: &String) -> Result<(usize, usize), LoxError> {
        if let Some(&(pos, size)) = self.scope_variables.get(name) {
            Ok((pos, size))
        }
        else if let Some(parent) = self.parent {
            parent.get_var(name)
        }
        else {
            Err(LoxError::CompilationError(format!("Couldn't find variable {} in the current scope", name)))
        }
    }
}
