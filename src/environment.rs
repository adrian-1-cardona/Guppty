// === environment.rs ===
// an environment is a box of named variables!
// boxes can nest inside bigger boxes — that is how scopes work.
// inner boxes see outer boxes but outer boxes cannot peek inside. shhh!

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::Stmt;
use crate::value::Value;

/// a function remembers the box it was born in — that is a closure!
#[derive(Debug, Clone)]
pub struct GuppyFunction {
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
    pub closure: Rc<RefCell<Environment>>,
}

#[derive(Debug, Clone)]
pub struct Environment {
    pub values: HashMap<String, Value>,
    pub parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    /// make a brand new empty box with no parent
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Environment {
            values: HashMap::new(),
            parent: None,
        }))
    }

    /// make a smaller box that lives inside another box (a new scope!)
    pub fn with_parent(parent: Rc<RefCell<Environment>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Environment {
            values: HashMap::new(),
            parent: Some(parent),
        }))
    }

    /// put a new name in THIS box only
    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    /// find a name — look here first, then ask parent boxes up the chain
    pub fn get(&self, name: &str) -> Result<Value, String> {
        if let Some(value) = self.values.get(name) {
            return Ok(value.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow().get(name);
        }

        Err(format!("Variable '{}' is not defined yet!", name))
    }

    /// change a name that already exists somewhere in the chain
    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), String> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            return Ok(());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow_mut().assign(name, value);
        }

        Err(format!("Variable '{}' is not defined yet!", name))
    }

    /// does this name exist in this box or any parent?
    pub fn exists(&self, name: &str) -> bool {
        if self.values.contains_key(name) {
            return true;
        }

        if let Some(parent) = &self.parent {
            return parent.borrow().exists(name);
        }

        false
    }
}
