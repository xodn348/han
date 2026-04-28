use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use super::value::Value;

#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
    pub line: usize,
    pub stack_trace: Vec<String>,
}

impl RuntimeError {
    pub fn new(msg: impl Into<String>, line: usize) -> Self {
        Self {
            message: msg.into(),
            line,
            stack_trace: Vec::new(),
        }
    }

    pub fn with_frame(mut self, frame: String) -> Self {
        self.stack_trace.push(frame);
        self
    }
}

pub type EnvRef = Rc<RefCell<Environment>>;

#[derive(Debug)]
pub struct Environment {
    store: HashMap<String, Value>,
    consts: HashSet<String>,
    imported_paths: HashSet<String>,
    outer: Option<EnvRef>,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            consts: HashSet::new(),
            imported_paths: HashSet::new(),
            outer: None,
        }
    }

    pub fn new_ref() -> EnvRef {
        Rc::new(RefCell::new(Self::new()))
    }

    pub fn new_enclosed(outer: EnvRef) -> Self {
        Self {
            store: HashMap::new(),
            consts: HashSet::new(),
            imported_paths: HashSet::new(),
            outer: Some(outer),
        }
    }

    pub fn new_enclosed_ref(outer: EnvRef) -> EnvRef {
        Rc::new(RefCell::new(Self::new_enclosed(outer)))
    }

    pub fn has_imported_path(&self, path: &str) -> bool {
        if self.imported_paths.contains(path) {
            true
        } else if let Some(outer) = &self.outer {
            outer.borrow().has_imported_path(path)
        } else {
            false
        }
    }

    pub fn mark_imported_path(&mut self, path: &str) {
        if let Some(outer) = self.outer.clone() {
            outer.borrow_mut().mark_imported_path(path);
        } else {
            self.imported_paths.insert(path.to_string());
        }
    }

    pub fn unmark_imported_path(&mut self, path: &str) {
        if let Some(outer) = self.outer.clone() {
            outer.borrow_mut().unmark_imported_path(path);
        } else {
            self.imported_paths.remove(path);
        }
    }

    pub fn set_const(&mut self, name: String, val: Value) {
        self.consts.insert(name.clone());
        self.store.insert(name, val);
    }

    pub fn is_const(&self, name: &str) -> bool {
        if self.consts.contains(name) {
            true
        } else if let Some(outer) = &self.outer {
            outer.borrow().is_const(name)
        } else {
            false
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        match self.store.get(name) {
            Some(v) => Some(v.clone()),
            None => self.outer.as_ref()?.borrow().get(name),
        }
    }

    pub fn set(&mut self, name: String, val: Value) {
        self.store.insert(name, val);
    }

    pub fn update(&mut self, name: &str, val: Value) -> bool {
        if self.store.contains_key(name) {
            self.store.insert(name.to_string(), val);
            true
        } else if let Some(outer) = self.outer.clone() {
            outer.borrow_mut().update(name, val)
        } else {
            false
        }
    }

    #[allow(dead_code)]
    pub fn collect_functions(&self) -> Vec<(String, Value)> {
        let mut funcs: Vec<(String, Value)> = self
            .store
            .iter()
            .filter(|(_, v)| matches!(v, Value::Function { .. }))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        if let Some(outer) = &self.outer {
            for (k, v) in outer.borrow().collect_functions() {
                if !funcs.iter().any(|(name, _)| name == &k) {
                    funcs.push((k, v));
                }
            }
        }
        funcs
    }

    pub fn snapshot(&self) -> Vec<(String, Value)> {
        let mut all: Vec<(String, Value)> = self
            .store
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        if let Some(outer) = &self.outer {
            for (k, v) in outer.borrow().snapshot() {
                if !all.iter().any(|(name, _)| name == &k) {
                    all.push((k, v));
                }
            }
        }
        all
    }
}
