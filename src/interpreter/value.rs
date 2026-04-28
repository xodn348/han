use super::env::EnvRef;
use crate::ast::{Stmt, Type};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Void,
    Function {
        params: Vec<(String, Type)>,
        body: Rc<Vec<Stmt>>,
    },
    Closure {
        params: Vec<(String, Option<Type>)>,
        body: Rc<Vec<Stmt>>,
        captured: EnvRef,
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

pub enum Signal {
    Return(Value),
    Break,
    Continue,
}
