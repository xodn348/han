use super::value::Value;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub type EnvRef = Rc<RefCell<Environment>>;

#[derive(Debug)]
pub struct Environment {
    store: HashMap<String, Value>,
    consts: HashSet<String>,
    imported_paths: HashSet<String>,
    outer: Option<EnvRef>,
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

    pub fn new_enclosed(outer: EnvRef) -> EnvRef {
        Rc::new(RefCell::new(Self {
            store: HashMap::new(),
            consts: HashSet::new(),
            imported_paths: HashSet::new(),
            outer: Some(outer),
        }))
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
        if let Some(outer) = &self.outer {
            outer.borrow_mut().mark_imported_path(path);
        } else {
            self.imported_paths.insert(path.to_string());
        }
    }

    pub fn unmark_imported_path(&mut self, path: &str) {
        if let Some(outer) = &self.outer {
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
        if let Some(v) = self.store.get(name) {
            return Some(v.clone());
        }
        self.outer.as_ref().and_then(|o| o.borrow().get(name))
    }

    #[allow(dead_code)]
    pub fn get_local(&self, name: &str) -> Option<&Value> {
        self.store.get(name)
    }

    pub fn set(&mut self, name: String, val: Value) {
        self.store.insert(name, val);
    }

    pub fn update(&mut self, name: &str, val: Value) -> bool {
        if self.store.contains_key(name) {
            self.store.insert(name.to_string(), val);
            true
        } else if let Some(outer) = &self.outer {
            outer.borrow_mut().update(name, val)
        } else {
            false
        }
    }

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
