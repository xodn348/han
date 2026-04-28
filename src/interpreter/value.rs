use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::{Stmt, Type};

use super::env::EnvRef;

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
    Closure {
        params: Vec<(String, Option<Type>)>,
        body: Vec<Stmt>,
        env: EnvRef,
    },
    Array(Rc<RefCell<Vec<Value>>>),
    Struct {
        name: String,
        fields: Rc<RefCell<HashMap<String, Value>>>,
    },
    Tuple(Vec<Value>),
    Map(Rc<RefCell<Vec<(Value, Value)>>>),
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
            Value::Closure { .. } => write!(f, "<람다>"),
            Value::Array(arr) => {
                let arr = arr.borrow();
                let items: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", items.join(", "))
            }
            Value::Tuple(vals) => {
                let items: Vec<String> = vals.iter().map(|v| v.to_string()).collect();
                write!(f, "({})", items.join(", "))
            }
            Value::Map(entries) => {
                let entries = entries.borrow();
                let items: Vec<String> = entries
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect();
                write!(f, "{{{}}}", items.join(", "))
            }
            Value::Struct { name, fields } => {
                let fields = fields.borrow();
                let mut pairs: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect();
                pairs.sort();
                write!(f, "{} {{ {} }}", name, pairs.join(", "))
            }
        }
    }
}

pub(crate) fn json_to_value(json: &serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Void,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else {
                Value::Float(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::String(s) => Value::Str(s.clone()),
        serde_json::Value::Array(arr) => {
            let vals: Vec<Value> = arr.iter().map(json_to_value).collect();
            Value::Array(Rc::new(RefCell::new(vals)))
        }
        serde_json::Value::Object(map) => {
            let pairs: Vec<(Value, Value)> = map
                .iter()
                .map(|(k, v)| (Value::Str(k.clone()), json_to_value(v)))
                .collect();
            Value::Map(Rc::new(RefCell::new(pairs)))
        }
    }
}

pub(crate) fn value_to_json(val: &Value) -> serde_json::Value {
    match val {
        Value::Int(n) => serde_json::Value::Number((*n).into()),
        Value::Float(f) => serde_json::json!(*f),
        Value::Str(s) => serde_json::Value::String(s.clone()),
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Void => serde_json::Value::Null,
        Value::Array(arr) => {
            let arr = arr.borrow();
            serde_json::Value::Array(arr.iter().map(value_to_json).collect())
        }
        Value::Map(map) => {
            let map = map.borrow();
            let mut obj = serde_json::Map::new();
            for (k, v) in map.iter() {
                obj.insert(k.to_string(), value_to_json(v));
            }
            serde_json::Value::Object(obj)
        }
        _ => serde_json::Value::Null,
    }
}
