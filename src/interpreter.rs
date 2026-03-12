#![allow(dead_code, unused)]

use crate::ast::{Stmt, Type};
use std::collections::HashMap;

/// 런타임 값
#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Void,
    Function {
        params: Vec<(String, Type)>,
        body: Vec<Stmt>,
    },
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::Str(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", if *b { "참" } else { "거짓" }),
            Value::Void => write!(f, "없음"),
            Value::Function { .. } => write!(f, "<함수>"),
        }
    }
}

/// 런타임 에러
#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
}

impl RuntimeError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

/// 변수 환경 (스코프 체인)
pub struct Environment {
    store: HashMap<String, Value>,
    outer: Option<Box<Environment>>,
}

impl Environment {
    /// 새 최상위 환경
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            outer: None,
        }
    }

    /// 새 중첩 환경 (함수 호출 시)
    pub fn new_enclosed(outer: Environment) -> Self {
        Self {
            store: HashMap::new(),
            outer: Some(Box::new(outer)),
        }
    }

    /// 변수 조회 (현재 → 외부 순)
    pub fn get(&self, name: &str) -> Option<Value> {
        match self.store.get(name) {
            Some(v) => Some(v.clone()),
            None => self.outer.as_ref()?.get(name),
        }
    }

    /// 변수 설정 (현재 스코프)
    pub fn set(&mut self, name: String, val: Value) {
        self.store.insert(name, val);
    }

    /// 변수 업데이트 (존재하는 스코프에서)
    pub fn update(&mut self, name: &str, val: Value) -> bool {
        if self.store.contains_key(name) {
            self.store.insert(name.to_string(), val);
            true
        } else if let Some(outer) = &mut self.outer {
            outer.update(name, val)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_set_get() {
        let mut env = Environment::new();
        env.set("나이".to_string(), Value::Int(20));
        assert!(matches!(env.get("나이"), Some(Value::Int(20))));
    }

    #[test]
    fn test_env_scope_chain() {
        let mut outer = Environment::new();
        outer.set("x".to_string(), Value::Int(10));
        let inner = Environment::new_enclosed(outer);
        assert!(matches!(inner.get("x"), Some(Value::Int(10))));
        assert!(inner.get("y").is_none());
    }

    #[test]
    fn test_value_display() {
        assert_eq!(Value::Int(42).to_string(), "42");
        assert_eq!(Value::Bool(true).to_string(), "참");
        assert_eq!(Value::Bool(false).to_string(), "거짓");
        assert_eq!(Value::Void.to_string(), "없음");
    }

    #[test]
    fn test_env_update() {
        let mut env = Environment::new();
        env.set("x".to_string(), Value::Int(1));
        env.update("x", Value::Int(2));
        assert!(matches!(env.get("x"), Some(Value::Int(2))));
    }
}
