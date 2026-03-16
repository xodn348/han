use crate::ast::{BinaryOpKind, Expr, Pattern, Program, Stmt, StmtKind, Type, UnaryOpKind};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{self, BufRead};
use std::rc::Rc;

thread_local! {
    static OUTPUT_BUFFER: RefCell<Option<String>> = const { RefCell::new(None) };
}

pub fn capture_start() {
    OUTPUT_BUFFER.with(|b| *b.borrow_mut() = Some(String::new()));
}

pub fn capture_flush() -> String {
    OUTPUT_BUFFER.with(|b| b.borrow_mut().take().unwrap_or_default())
}

fn output_line(s: &str) {
    OUTPUT_BUFFER.with(|b| {
        if let Some(buf) = b.borrow_mut().as_mut() {
            buf.push_str(s);
            buf.push('\n');
        } else {
            println!("{}", s);
        }
    });
}

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
        captured: Vec<(String, Value)>,
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

pub struct Environment {
    store: HashMap<String, Value>,
    outer: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            outer: None,
        }
    }

    #[cfg(test)]
    pub fn new_enclosed(outer: Environment) -> Self {
        Self {
            store: HashMap::new(),
            outer: Some(Box::new(outer)),
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        match self.store.get(name) {
            Some(v) => Some(v.clone()),
            None => self.outer.as_ref()?.get(name),
        }
    }

    pub fn set(&mut self, name: String, val: Value) {
        self.store.insert(name, val);
    }

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

    pub fn collect_functions(&self) -> Vec<(String, Value)> {
        let mut funcs: Vec<(String, Value)> = self
            .store
            .iter()
            .filter(|(_, v)| matches!(v, Value::Function { .. }))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        if let Some(outer) = &self.outer {
            for (k, v) in outer.collect_functions() {
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
            for (k, v) in outer.snapshot() {
                if !all.iter().any(|(name, _)| name == &k) {
                    all.push((k, v));
                }
            }
        }
        all
    }
}

pub enum Signal {
    Return(Value),
    Break,
    Continue,
}

pub fn eval_expr(expr: &Expr, env: &mut Environment, line: usize) -> Result<Value, RuntimeError> {
    match expr {
        Expr::IntLiteral(n) => Ok(Value::Int(*n)),
        Expr::FloatLiteral(f) => Ok(Value::Float(*f)),
        Expr::StringLiteral(s) => Ok(Value::Str(s.clone())),
        Expr::BoolLiteral(b) => Ok(Value::Bool(*b)),
        Expr::NullLiteral => Ok(Value::Void),

        Expr::Identifier(name) => env
            .get(name)
            .ok_or_else(|| RuntimeError::new(format!("정의되지 않은 변수: {}", name), line)),

        Expr::Assign { name, value } => {
            let val = eval_expr(value, env, line)?;
            if !env.update(name, val.clone()) {
                env.set(name.clone(), val.clone());
            }
            Ok(val)
        }

        Expr::BinaryOp { op, left, right } => {
            let lv = eval_expr(left, env, line)?;
            let rv = eval_expr(right, env, line)?;
            eval_binary_op(op, lv, rv, line)
        }

        Expr::UnaryOp { op, expr } => {
            let val = eval_expr(expr, env, line)?;
            match op {
                UnaryOpKind::Neg => match val {
                    Value::Int(n) => Ok(Value::Int(-n)),
                    Value::Float(f) => Ok(Value::Float(-f)),
                    _ => Err(RuntimeError::new(
                        "단항 음수는 정수/실수에만 적용 가능",
                        line,
                    )),
                },
                UnaryOpKind::Not => match val {
                    Value::Bool(b) => Ok(Value::Bool(!b)),
                    _ => Err(RuntimeError::new("논리 부정은 불 값에만 적용 가능", line)),
                },
            }
        }

        Expr::Call { name, args } => {
            if name == "출력" {
                let mut parts = Vec::new();
                for arg in args {
                    let v = eval_expr(arg, env, line)?;
                    parts.push(v.to_string());
                }
                output_line(&parts.join(" "));
                return Ok(Value::Void);
            }

            if name == "입력" {
                let stdin = io::stdin();
                let mut buf = String::new();
                stdin
                    .lock()
                    .read_line(&mut buf)
                    .map_err(|e| RuntimeError::new(format!("입력 오류: {}", e), line))?;
                return Ok(Value::Str(buf.trim_end_matches('\n').to_string()));
            }

            if let Some(result) = eval_builtin_math(name, args, env, line)? {
                return Ok(result);
            }

            if let Some(result) = eval_builtin_stdlib(name, args, env, line)? {
                return Ok(result);
            }

            if name == "사전" {
                let mut pairs = Vec::new();
                let mut i = 0;
                while i + 1 < args.len() {
                    let key = eval_expr(&args[i], env, line)?;
                    let val = eval_expr(&args[i + 1], env, line)?;
                    pairs.push((key, val));
                    i += 2;
                }
                return Ok(Value::Map(Rc::new(RefCell::new(pairs))));
            }

            if let Some(result) = eval_builtin_io(name, args, env, line)? {
                return Ok(result);
            }

            let func_val = env
                .get(name)
                .ok_or_else(|| RuntimeError::new(format!("정의되지 않은 함수: {}", name), line))?;

            match func_val {
                Value::Function { params, body } => {
                    if args.len() != params.len() {
                        return Err(RuntimeError::new(
                            format!(
                                "함수 '{}': 인자 수 불일치 (기대 {}, 실제 {})",
                                name,
                                params.len(),
                                args.len()
                            ),
                            line,
                        ));
                    }

                    let mut arg_vals = Vec::new();
                    for arg in args {
                        arg_vals.push(eval_expr(arg, env, line)?);
                    }

                    let mut func_env = Environment::new();
                    for (k, v) in env.snapshot() {
                        func_env.set(k, v);
                    }
                    for ((param_name, _ty), val) in params.iter().zip(arg_vals) {
                        func_env.set(param_name.clone(), val);
                    }

                    match eval_block(&body, &mut func_env) {
                        Ok(Some(Signal::Return(v))) => Ok(v),
                        Ok(_) => Ok(Value::Void),
                        Err(e) => Err(e.with_frame(format!("  함수 '{}' ({}번째 줄)", name, line))),
                    }
                }
                Value::Closure {
                    params,
                    body,
                    captured,
                } => {
                    if args.len() != params.len() {
                        return Err(RuntimeError::new(
                            format!(
                                "람다 '{}': 인자 수 불일치 (기대 {}, 실제 {})",
                                name,
                                params.len(),
                                args.len()
                            ),
                            line,
                        ));
                    }
                    let mut arg_vals = Vec::new();
                    for arg in args {
                        arg_vals.push(eval_expr(arg, env, line)?);
                    }
                    let mut closure_env = Environment::new();
                    for (k, v) in &captured {
                        closure_env.set(k.clone(), v.clone());
                    }
                    for ((param_name, _), val) in params.iter().zip(arg_vals) {
                        closure_env.set(param_name.clone(), val);
                    }
                    match eval_block(&body, &mut closure_env)? {
                        Some(Signal::Return(v)) => Ok(v),
                        _ => Ok(Value::Void),
                    }
                }
                _ => Err(RuntimeError::new(
                    format!("'{}' 는 함수가 아닙니다", name),
                    line,
                )),
            }
        }

        Expr::ArrayLiteral(elems) => {
            let mut vals = Vec::new();
            for e in elems {
                vals.push(eval_expr(e, env, line)?);
            }
            Ok(Value::Array(Rc::new(RefCell::new(vals))))
        }

        Expr::Index { object, index } => {
            let obj = eval_expr(object, env, line)?;
            let idx = eval_expr(index, env, line)?;
            match (obj, idx) {
                (Value::Array(arr), Value::Int(i)) => {
                    let arr = arr.borrow();
                    let len = arr.len() as i64;
                    let i = if i < 0 { len + i } else { i };
                    if i < 0 || i >= len {
                        Err(RuntimeError::new(
                            format!("인덱스 범위 초과: {} (길이 {})", i, len),
                            line,
                        ))
                    } else {
                        Ok(arr[i as usize].clone())
                    }
                }
                (Value::Str(s), Value::Int(i)) => {
                    let chars: Vec<char> = s.chars().collect();
                    let len = chars.len() as i64;
                    let i = if i < 0 { len + i } else { i };
                    if i < 0 || i >= len {
                        Err(RuntimeError::new(
                            format!("문자열 인덱스 범위 초과: {}", i),
                            line,
                        ))
                    } else {
                        Ok(Value::Str(chars[i as usize].to_string()))
                    }
                }
                (Value::Map(map), key) => {
                    let map = map.borrow();
                    for (k, v) in map.iter() {
                        if values_equal(k, &key) {
                            return Ok(v.clone());
                        }
                    }
                    Err(RuntimeError::new(
                        format!("사전에 키가 없음: {}", key),
                        line,
                    ))
                }
                _ => Err(RuntimeError::new("인덱싱 불가 타입", line)),
            }
        }

        Expr::IndexAssign {
            object,
            index,
            value,
        } => {
            let obj = eval_expr(object, env, line)?;
            let idx = eval_expr(index, env, line)?;
            let val = eval_expr(value, env, line)?;
            match (obj, idx) {
                (Value::Array(arr), Value::Int(i)) => {
                    let mut arr = arr.borrow_mut();
                    let len = arr.len() as i64;
                    let i = if i < 0 { len + i } else { i };
                    if i < 0 || i >= len {
                        return Err(RuntimeError::new(format!("인덱스 범위 초과: {}", i), line));
                    }
                    arr[i as usize] = val.clone();
                    Ok(val)
                }
                (Value::Map(map), key) => {
                    let mut map = map.borrow_mut();
                    for entry in map.iter_mut() {
                        if values_equal(&entry.0, &key) {
                            entry.1 = val.clone();
                            return Ok(val);
                        }
                    }
                    map.push((key, val.clone()));
                    Ok(val)
                }
                _ => Err(RuntimeError::new("인덱스 할당: 배열/사전 타입 필요", line)),
            }
        }

        Expr::MethodCall {
            object,
            method,
            args,
        } => {
            let obj = eval_expr(object, env, line)?;
            eval_method(obj, method, args, env, line)
        }

        Expr::FieldAccess { object, field } => {
            let obj = eval_expr(object, env, line)?;
            match obj {
                Value::Struct { fields, .. } => {
                    fields.borrow().get(field.as_str()).cloned().ok_or_else(|| {
                        RuntimeError::new(format!("존재하지 않는 필드: {}", field), line)
                    })
                }
                _ => Err(RuntimeError::new("필드 접근: 구조체 타입 필요", line)),
            }
        }

        Expr::FieldAssign {
            object,
            field,
            value,
        } => {
            let obj = eval_expr(object, env, line)?;
            let val = eval_expr(value, env, line)?;
            match obj {
                Value::Struct { fields, .. } => {
                    fields.borrow_mut().insert(field.clone(), val.clone());
                    Ok(val)
                }
                _ => Err(RuntimeError::new("필드 할당: 구조체 타입 필요", line)),
            }
        }

        Expr::StructLiteral { name, fields } => {
            let mut map = HashMap::new();
            for (fname, fexpr) in fields {
                map.insert(fname.clone(), eval_expr(fexpr, env, line)?);
            }
            Ok(Value::Struct {
                name: name.clone(),
                fields: Rc::new(RefCell::new(map)),
            })
        }

        Expr::Lambda { params, body } => {
            let captured = env.snapshot();
            Ok(Value::Closure {
                params: params.clone(),
                body: body.clone(),
                captured,
            })
        }

        Expr::Range { start, end } => {
            let s = match eval_expr(start, env, line)? {
                Value::Int(n) => n,
                _ => return Err(RuntimeError::new("범위: 정수 필요", line)),
            };
            let e = match eval_expr(end, env, line)? {
                Value::Int(n) => n,
                _ => return Err(RuntimeError::new("범위: 정수 필요", line)),
            };
            let vals: Vec<Value> = (s..e).map(Value::Int).collect();
            Ok(Value::Array(Rc::new(RefCell::new(vals))))
        }

        Expr::TupleLiteral(elems) => {
            let mut vals = Vec::new();
            for e in elems {
                vals.push(eval_expr(e, env, line)?);
            }
            Ok(Value::Tuple(vals))
        }

        Expr::TupleIndex { object, index } => {
            let obj = eval_expr(object, env, line)?;
            match obj {
                Value::Tuple(vals) => {
                    if *index >= vals.len() {
                        Err(RuntimeError::new(
                            format!("튜플 인덱스 범위 초과: {} (길이 {})", index, vals.len()),
                            line,
                        ))
                    } else {
                        Ok(vals[*index].clone())
                    }
                }
                _ => Err(RuntimeError::new("튜플 인덱싱: 튜플 타입 필요", line)),
            }
        }

        Expr::MapLiteral(entries) => {
            let mut pairs = Vec::new();
            for (k, v) in entries {
                let key = eval_expr(k, env, line)?;
                let val = eval_expr(v, env, line)?;
                pairs.push((key, val));
            }
            Ok(Value::Map(Rc::new(RefCell::new(pairs))))
        }
    }
}

fn eval_method(
    obj: Value,
    method: &str,
    args: &[Expr],
    env: &mut Environment,
    line: usize,
) -> Result<Value, RuntimeError> {
    let arg_vals: Vec<Value> = args
        .iter()
        .map(|a| eval_expr(a, env, line))
        .collect::<Result<_, _>>()?;

    match obj {
        Value::Array(arr) => match method {
            "추가" => {
                let val = arg_vals
                    .into_iter()
                    .next()
                    .ok_or_else(|| RuntimeError::new("추가: 인자 1개 필요", line))?;
                arr.borrow_mut().push(val);
                Ok(Value::Void)
            }
            "삭제" => {
                let idx = match arg_vals.first() {
                    Some(Value::Int(i)) => *i,
                    _ => return Err(RuntimeError::new("삭제: 정수 인덱스 필요", line)),
                };
                let mut arr = arr.borrow_mut();
                let len = arr.len() as i64;
                let idx = if idx < 0 { len + idx } else { idx };
                if idx < 0 || idx >= len {
                    return Err(RuntimeError::new(
                        format!("삭제: 인덱스 범위 초과 {}", idx),
                        line,
                    ));
                }
                Ok(arr.remove(idx as usize))
            }
            "길이" => Ok(Value::Int(arr.borrow().len() as i64)),
            "포함" => {
                let val = arg_vals
                    .first()
                    .ok_or_else(|| RuntimeError::new("포함: 인자 1개 필요", line))?;
                let found = arr.borrow().iter().any(|v| values_equal(v, val));
                Ok(Value::Bool(found))
            }
            "역순" => {
                let mut v = arr.borrow().clone();
                v.reverse();
                Ok(Value::Array(Rc::new(RefCell::new(v))))
            }
            "정렬" => {
                let mut v = arr.borrow().clone();
                v.sort_by(|a, b| match (a, b) {
                    (Value::Int(x), Value::Int(y)) => x.cmp(y),
                    (Value::Float(x), Value::Float(y)) => {
                        x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    (Value::Str(x), Value::Str(y)) => x.cmp(y),
                    _ => std::cmp::Ordering::Equal,
                });
                Ok(Value::Array(Rc::new(RefCell::new(v))))
            }
            "합치기" => {
                let sep = match arg_vals.first() {
                    Some(Value::Str(s)) => s.clone(),
                    _ => "".to_string(),
                };
                let parts: Vec<String> = arr.borrow().iter().map(|v| v.to_string()).collect();
                Ok(Value::Str(parts.join(&sep)))
            }
            _ => Err(RuntimeError::new(
                format!("배열 메서드 없음: {}", method),
                line,
            )),
        },
        Value::Str(s) => match method {
            "길이" => Ok(Value::Int(s.chars().count() as i64)),
            "분리" => {
                let sep = match arg_vals.first() {
                    Some(Value::Str(sep)) => sep.clone(),
                    _ => " ".to_string(),
                };
                let parts: Vec<Value> = s
                    .split(sep.as_str())
                    .map(|p| Value::Str(p.to_string()))
                    .collect();
                Ok(Value::Array(Rc::new(RefCell::new(parts))))
            }
            "포함" => {
                let needle = match arg_vals.first() {
                    Some(Value::Str(n)) => n.clone(),
                    _ => return Err(RuntimeError::new("포함: 문자열 인자 필요", line)),
                };
                Ok(Value::Bool(s.contains(needle.as_str())))
            }
            "바꾸기" => {
                let from = match arg_vals.first() {
                    Some(Value::Str(f)) => f.clone(),
                    _ => return Err(RuntimeError::new("바꾸기: 문자열 인자 2개 필요", line)),
                };
                let to = match arg_vals.get(1) {
                    Some(Value::Str(t)) => t.clone(),
                    _ => return Err(RuntimeError::new("바꾸기: 문자열 인자 2개 필요", line)),
                };
                Ok(Value::Str(s.replace(from.as_str(), to.as_str())))
            }
            "앞뒤공백제거" => Ok(Value::Str(s.trim().to_string())),
            "대문자" => Ok(Value::Str(s.to_uppercase())),
            "소문자" => Ok(Value::Str(s.to_lowercase())),
            "시작" => {
                let prefix = match arg_vals.first() {
                    Some(Value::Str(p)) => p.clone(),
                    _ => return Err(RuntimeError::new("시작: 문자열 인자 필요", line)),
                };
                Ok(Value::Bool(s.starts_with(prefix.as_str())))
            }
            "끝" => {
                let suffix = match arg_vals.first() {
                    Some(Value::Str(p)) => p.clone(),
                    _ => return Err(RuntimeError::new("끝: 문자열 인자 필요", line)),
                };
                Ok(Value::Bool(s.ends_with(suffix.as_str())))
            }
            _ => Err(RuntimeError::new(
                format!("문자열 메서드 없음: {}", method),
                line,
            )),
        },
        Value::Map(map) => match method {
            "키목록" => {
                let keys: Vec<Value> = map.borrow().iter().map(|(k, _)| k.clone()).collect();
                Ok(Value::Array(Rc::new(RefCell::new(keys))))
            }
            "값목록" => {
                let vals: Vec<Value> = map.borrow().iter().map(|(_, v)| v.clone()).collect();
                Ok(Value::Array(Rc::new(RefCell::new(vals))))
            }
            "길이" => Ok(Value::Int(map.borrow().len() as i64)),
            "포함" => {
                let key = arg_vals
                    .first()
                    .ok_or_else(|| RuntimeError::new("포함: 키 인자 필요", line))?;
                let found = map.borrow().iter().any(|(k, _)| values_equal(k, key));
                Ok(Value::Bool(found))
            }
            "삭제" => {
                let key = arg_vals
                    .first()
                    .ok_or_else(|| RuntimeError::new("삭제: 키 인자 필요", line))?;
                let mut map = map.borrow_mut();
                let pos = map.iter().position(|(k, _)| values_equal(k, key));
                if let Some(i) = pos {
                    let (_, v) = map.remove(i);
                    Ok(v)
                } else {
                    Err(RuntimeError::new("삭제: 키 없음", line))
                }
            }
            _ => Err(RuntimeError::new(
                format!("사전 메서드 없음: {}", method),
                line,
            )),
        },
        Value::Struct {
            name: struct_name,
            fields,
        } => {
            let method_key = format!("{}::{}", struct_name, method);
            if let Some(func_val) = env.get(&method_key) {
                match func_val {
                    Value::Function { params, body } => {
                        let mut method_env = Environment::new();
                        for (fname, fval) in env.collect_functions() {
                            method_env.set(fname, fval);
                        }
                        method_env.set(
                            "자신".to_string(),
                            Value::Struct {
                                name: struct_name.clone(),
                                fields: fields.clone(),
                            },
                        );
                        let non_self_params: Vec<_> =
                            params.iter().filter(|(n, _)| n != "자신").collect();
                        for ((pname, _), val) in non_self_params.iter().zip(arg_vals) {
                            method_env.set(pname.clone(), val);
                        }
                        match eval_block(&body, &mut method_env)? {
                            Some(Signal::Return(v)) => Ok(v),
                            _ => Ok(Value::Void),
                        }
                    }
                    _ => Err(RuntimeError::new(
                        format!("'{}' 는 함수가 아닙니다", method_key),
                        line,
                    )),
                }
            } else {
                Err(RuntimeError::new(
                    format!("구조체 '{}' 에 메서드 '{}' 없음", struct_name, method),
                    line,
                ))
            }
        }
        _ => Err(RuntimeError::new(
            format!("메서드 '{}' 호출 불가 타입", method),
            line,
        )),
    }
}

fn json_to_value(json: &serde_json::Value) -> Value {
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

fn value_to_json(val: &Value) -> serde_json::Value {
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

fn eval_builtin_stdlib(
    name: &str,
    args: &[Expr],
    env: &mut Environment,
    line: usize,
) -> Result<Option<Value>, RuntimeError> {
    match name {
        "제이슨_파싱" => {
            if args.len() != 1 {
                return Err(RuntimeError::new("제이슨_파싱: 문자열 인자 필요", line));
            }
            let s = match eval_expr(&args[0], env, line)? {
                Value::Str(s) => s,
                _ => return Err(RuntimeError::new("제이슨_파싱: 문자열 필요", line)),
            };
            let json: serde_json::Value = serde_json::from_str(&s)
                .map_err(|e| RuntimeError::new(format!("JSON 파싱 오류: {}", e), line))?;
            Ok(Some(json_to_value(&json)))
        }
        "제이슨_생성" => {
            if args.len() != 1 {
                return Err(RuntimeError::new("제이슨_생성: 인자 1개 필요", line));
            }
            let val = eval_expr(&args[0], env, line)?;
            let json = value_to_json(&val);
            Ok(Some(Value::Str(json.to_string())))
        }
        "제이슨_예쁘게" => {
            if args.len() != 1 {
                return Err(RuntimeError::new("제이슨_예쁘게: 인자 1개 필요", line));
            }
            let val = eval_expr(&args[0], env, line)?;
            let json = value_to_json(&val);
            let pretty = serde_json::to_string_pretty(&json)
                .map_err(|e| RuntimeError::new(format!("JSON 변환 오류: {}", e), line))?;
            Ok(Some(Value::Str(pretty)))
        }
        "HTTP_가져오기" => {
            #[cfg(feature = "native")]
            {
                if args.len() != 1 {
                    return Err(RuntimeError::new("HTTP_가져오기: URL 인자 필요", line));
                }
                let url = match eval_expr(&args[0], env, line)? {
                    Value::Str(s) => s,
                    _ => return Err(RuntimeError::new("HTTP_가져오기: 문자열 URL 필요", line)),
                };
                let body = reqwest::blocking::get(&url)
                    .map_err(|e| RuntimeError::new(format!("HTTP 오류: {}", e), line))?
                    .text()
                    .map_err(|e| RuntimeError::new(format!("HTTP 응답 읽기 오류: {}", e), line))?;
                return Ok(Some(Value::Str(body)));
            }
            #[cfg(not(feature = "native"))]
            return Err(RuntimeError::new(
                "HTTP_가져오기: 플레이그라운드에서 미지원",
                line,
            ));
        }
        "HTTP_보내기" => {
            #[cfg(feature = "native")]
            {
                if args.len() < 2 {
                    return Err(RuntimeError::new("HTTP_보내기: URL, 본문 인자 필요", line));
                }
                let url = match eval_expr(&args[0], env, line)? {
                    Value::Str(s) => s,
                    _ => return Err(RuntimeError::new("HTTP_보내기: 문자열 URL 필요", line)),
                };
                let body_val = eval_expr(&args[1], env, line)?;
                let body_str = match &body_val {
                    Value::Str(s) => s.clone(),
                    _ => value_to_json(&body_val).to_string(),
                };
                let client = reqwest::blocking::Client::new();
                let resp = client
                    .post(&url)
                    .header("Content-Type", "application/json")
                    .body(body_str)
                    .send()
                    .map_err(|e| RuntimeError::new(format!("HTTP POST 오류: {}", e), line))?
                    .text()
                    .map_err(|e| RuntimeError::new(format!("HTTP 응답 읽기 오류: {}", e), line))?;
                return Ok(Some(Value::Str(resp)));
            }
            #[cfg(not(feature = "native"))]
            return Err(RuntimeError::new(
                "HTTP_보내기: 플레이그라운드에서 미지원",
                line,
            ));
        }
        "정규식_찾기" => {
            if args.len() != 2 {
                return Err(RuntimeError::new(
                    "정규식_찾기: 패턴, 텍스트 인자 필요",
                    line,
                ));
            }
            let pattern = match eval_expr(&args[0], env, line)? {
                Value::Str(s) => s,
                _ => return Err(RuntimeError::new("정규식_찾기: 문자열 패턴 필요", line)),
            };
            let text = match eval_expr(&args[1], env, line)? {
                Value::Str(s) => s,
                _ => return Err(RuntimeError::new("정규식_찾기: 문자열 텍스트 필요", line)),
            };
            let re = regex::Regex::new(&pattern)
                .map_err(|e| RuntimeError::new(format!("정규식 오류: {}", e), line))?;
            let matches: Vec<Value> = re
                .find_iter(&text)
                .map(|m| Value::Str(m.as_str().to_string()))
                .collect();
            Ok(Some(Value::Array(Rc::new(RefCell::new(matches)))))
        }
        "정규식_일치" => {
            if args.len() != 2 {
                return Err(RuntimeError::new(
                    "정규식_일치: 패턴, 텍스트 인자 필요",
                    line,
                ));
            }
            let pattern = match eval_expr(&args[0], env, line)? {
                Value::Str(s) => s,
                _ => return Err(RuntimeError::new("정규식_일치: 문자열 패턴 필요", line)),
            };
            let text = match eval_expr(&args[1], env, line)? {
                Value::Str(s) => s,
                _ => return Err(RuntimeError::new("정규식_일치: 문자열 텍스트 필요", line)),
            };
            let re = regex::Regex::new(&pattern)
                .map_err(|e| RuntimeError::new(format!("정규식 오류: {}", e), line))?;
            Ok(Some(Value::Bool(re.is_match(&text))))
        }
        "정규식_바꾸기" => {
            if args.len() != 3 {
                return Err(RuntimeError::new(
                    "정규식_바꾸기: 패턴, 텍스트, 대체 인자 필요",
                    line,
                ));
            }
            let pattern = match eval_expr(&args[0], env, line)? {
                Value::Str(s) => s,
                _ => return Err(RuntimeError::new("정규식_바꾸기: 문자열 패턴 필요", line)),
            };
            let text = match eval_expr(&args[1], env, line)? {
                Value::Str(s) => s,
                _ => return Err(RuntimeError::new("정규식_바꾸기: 문자열 텍스트 필요", line)),
            };
            let replacement = match eval_expr(&args[2], env, line)? {
                Value::Str(s) => s,
                _ => return Err(RuntimeError::new("정규식_바꾸기: 문자열 대체 필요", line)),
            };
            let re = regex::Regex::new(&pattern)
                .map_err(|e| RuntimeError::new(format!("정규식 오류: {}", e), line))?;
            Ok(Some(Value::Str(
                re.replace_all(&text, replacement.as_str()).to_string(),
            )))
        }
        "현재시간" => {
            let now = chrono::Local::now();
            Ok(Some(Value::Str(
                now.format("%Y-%m-%d %H:%M:%S").to_string(),
            )))
        }
        "현재날짜" => {
            let now = chrono::Local::now();
            Ok(Some(Value::Str(now.format("%Y-%m-%d").to_string())))
        }
        "타임스탬프" => {
            let now = chrono::Utc::now();
            Ok(Some(Value::Int(now.timestamp())))
        }
        "명령인자" => {
            let args: Vec<Value> = std::env::args().skip(2).map(Value::Str).collect();
            Ok(Some(Value::Array(Rc::new(RefCell::new(args)))))
        }
        "환경변수" => {
            if args.len() != 1 {
                return Err(RuntimeError::new("환경변수: 변수명 인자 필요", line));
            }
            let var_name = match eval_expr(&args[0], env, line)? {
                Value::Str(s) => s,
                _ => return Err(RuntimeError::new("환경변수: 문자열 필요", line)),
            };
            match std::env::var(&var_name) {
                Ok(val) => Ok(Some(Value::Str(val))),
                Err(_) => Ok(Some(Value::Void)),
            }
        }
        "실행" => {
            #[cfg(feature = "native")]
            {
                if args.len() != 1 {
                    return Err(RuntimeError::new("실행: 명령어 문자열 필요", line));
                }
                let cmd = match eval_expr(&args[0], env, line)? {
                    Value::Str(s) => s,
                    _ => return Err(RuntimeError::new("실행: 문자열 필요", line)),
                };
                let output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .output()
                    .map_err(|e| RuntimeError::new(format!("실행 오류: {}", e), line))?;
                return Ok(Some(Value::Str(
                    String::from_utf8_lossy(&output.stdout).to_string(),
                )));
            }
            #[cfg(not(feature = "native"))]
            return Err(RuntimeError::new("실행: 플레이그라운드에서 미지원", line));
        }
        "잠자기" => {
            if args.len() != 1 {
                return Err(RuntimeError::new("잠자기: 밀리초 인자 필요", line));
            }
            let ms = match eval_expr(&args[0], env, line)? {
                Value::Int(n) => n as u64,
                _ => return Err(RuntimeError::new("잠자기: 정수 필요", line)),
            };
            std::thread::sleep(std::time::Duration::from_millis(ms));
            Ok(Some(Value::Void))
        }
        "타입" => {
            if args.len() != 1 {
                return Err(RuntimeError::new("타입: 인자 1개 필요", line));
            }
            let val = eval_expr(&args[0], env, line)?;
            let type_name = match val {
                Value::Int(_) => "정수",
                Value::Float(_) => "실수",
                Value::Str(_) => "문자열",
                Value::Bool(_) => "불",
                Value::Void => "없음",
                Value::Function { .. } => "함수",
                Value::Closure { .. } => "람다",
                Value::Array(_) => "배열",
                Value::Tuple(_) => "튜플",
                Value::Map(_) => "사전",
                Value::Struct { .. } => "구조체",
            };
            Ok(Some(Value::Str(type_name.to_string())))
        }
        _ => Ok(None),
    }
}

fn eval_builtin_io(
    name: &str,
    args: &[Expr],
    env: &mut Environment,
    line: usize,
) -> Result<Option<Value>, RuntimeError> {
    match name {
        "파일읽기" => {
            if args.len() != 1 {
                return Err(RuntimeError::new("파일읽기: 파일 경로 인자 필요", line));
            }
            let path = match eval_expr(&args[0], env, line)? {
                Value::Str(s) => s,
                _ => return Err(RuntimeError::new("파일읽기: 문자열 경로 필요", line)),
            };
            let content = std::fs::read_to_string(&path)
                .map_err(|e| RuntimeError::new(format!("파일읽기 실패 '{}': {}", path, e), line))?;
            Ok(Some(Value::Str(content)))
        }
        "파일쓰기" => {
            if args.len() != 2 {
                return Err(RuntimeError::new("파일쓰기: 경로, 내용 인자 필요", line));
            }
            let path = match eval_expr(&args[0], env, line)? {
                Value::Str(s) => s,
                _ => return Err(RuntimeError::new("파일쓰기: 문자열 경로 필요", line)),
            };
            let content = eval_expr(&args[1], env, line)?.to_string();
            std::fs::write(&path, &content)
                .map_err(|e| RuntimeError::new(format!("파일쓰기 실패 '{}': {}", path, e), line))?;
            Ok(Some(Value::Void))
        }
        "파일추가" => {
            if args.len() != 2 {
                return Err(RuntimeError::new("파일추가: 경로, 내용 인자 필요", line));
            }
            let path = match eval_expr(&args[0], env, line)? {
                Value::Str(s) => s,
                _ => return Err(RuntimeError::new("파일추가: 문자열 경로 필요", line)),
            };
            let content = eval_expr(&args[1], env, line)?.to_string();
            use std::io::Write as IoWrite;
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(&path)
                .map_err(|e| RuntimeError::new(format!("파일추가 실패 '{}': {}", path, e), line))?;
            file.write_all(content.as_bytes())
                .map_err(|e| RuntimeError::new(format!("파일추가 쓰기 실패: {}", e), line))?;
            Ok(Some(Value::Void))
        }
        "파일존재" => {
            if args.len() != 1 {
                return Err(RuntimeError::new("파일존재: 파일 경로 인자 필요", line));
            }
            let path = match eval_expr(&args[0], env, line)? {
                Value::Str(s) => s,
                _ => return Err(RuntimeError::new("파일존재: 문자열 경로 필요", line)),
            };
            Ok(Some(Value::Bool(std::path::Path::new(&path).exists())))
        }
        "출력오류" => {
            let mut parts = Vec::new();
            for arg in args {
                parts.push(eval_expr(arg, env, line)?.to_string());
            }
            eprintln!("{}", parts.join(" "));
            Ok(Some(Value::Void))
        }
        "형식" => {
            if args.is_empty() {
                return Err(RuntimeError::new("형식: 형식 문자열 인자 필요", line));
            }
            let template = match eval_expr(&args[0], env, line)? {
                Value::Str(s) => s,
                _ => return Err(RuntimeError::new("형식: 첫 인자는 문자열 필요", line)),
            };
            let mut positional = Vec::new();
            for arg in &args[1..] {
                positional.push(eval_expr(arg, env, line)?.to_string());
            }
            let result = if positional.is_empty() {
                let snapshot = env.snapshot();
                let mut out = template.clone();
                for (k, v) in &snapshot {
                    out = out.replace(&format!("{{{}}}", k), &v.to_string());
                }
                out
            } else {
                let mut out = template.clone();
                for (i, val) in positional.iter().enumerate() {
                    out = out.replace(&format!("{{{}}}", i), val);
                }
                out
            };
            Ok(Some(Value::Str(result)))
        }
        _ => Ok(None),
    }
}

fn eval_builtin_math(
    name: &str,
    args: &[Expr],
    env: &mut Environment,
    line: usize,
) -> Result<Option<Value>, RuntimeError> {
    match name {
        "제곱근" => {
            if args.len() != 1 {
                return Err(RuntimeError::new("제곱근: 인자 1개 필요", line));
            }
            let v = eval_expr(&args[0], env, line)?;
            match v {
                Value::Int(n) => Ok(Some(Value::Float((n as f64).sqrt()))),
                Value::Float(f) => Ok(Some(Value::Float(f.sqrt()))),
                _ => Err(RuntimeError::new("제곱근: 숫자 타입 필요", line)),
            }
        }
        "절댓값" => {
            if args.len() != 1 {
                return Err(RuntimeError::new("절댓값: 인자 1개 필요", line));
            }
            let v = eval_expr(&args[0], env, line)?;
            match v {
                Value::Int(n) => Ok(Some(Value::Int(n.abs()))),
                Value::Float(f) => Ok(Some(Value::Float(f.abs()))),
                _ => Err(RuntimeError::new("절댓값: 숫자 타입 필요", line)),
            }
        }
        "거듭제곱" => {
            if args.len() != 2 {
                return Err(RuntimeError::new(
                    "거듭제곱: 인자 2개 필요 (밑, 지수)",
                    line,
                ));
            }
            let base = eval_expr(&args[0], env, line)?;
            let exp = eval_expr(&args[1], env, line)?;
            match (base, exp) {
                (Value::Int(b), Value::Int(e)) => Ok(Some(Value::Float((b as f64).powf(e as f64)))),
                (Value::Float(b), Value::Float(e)) => Ok(Some(Value::Float(b.powf(e)))),
                (Value::Float(b), Value::Int(e)) => Ok(Some(Value::Float(b.powf(e as f64)))),
                (Value::Int(b), Value::Float(e)) => Ok(Some(Value::Float((b as f64).powf(e)))),
                _ => Err(RuntimeError::new("거듭제곱: 숫자 타입 필요", line)),
            }
        }
        "정수변환" => {
            if args.len() != 1 {
                return Err(RuntimeError::new("정수변환: 인자 1개 필요", line));
            }
            let v = eval_expr(&args[0], env, line)?;
            match v {
                Value::Int(n) => Ok(Some(Value::Int(n))),
                Value::Float(f) => Ok(Some(Value::Int(f as i64))),
                Value::Str(s) => s
                    .parse::<i64>()
                    .map(|n| Some(Value::Int(n)))
                    .map_err(|_| RuntimeError::new(format!("정수변환 실패: '{}'", s), line)),
                Value::Bool(b) => Ok(Some(Value::Int(if b { 1 } else { 0 }))),
                _ => Err(RuntimeError::new("정수변환: 변환 불가 타입", line)),
            }
        }
        "실수변환" => {
            if args.len() != 1 {
                return Err(RuntimeError::new("실수변환: 인자 1개 필요", line));
            }
            let v = eval_expr(&args[0], env, line)?;
            match v {
                Value::Int(n) => Ok(Some(Value::Float(n as f64))),
                Value::Float(f) => Ok(Some(Value::Float(f))),
                Value::Str(s) => s
                    .parse::<f64>()
                    .map(|f| Some(Value::Float(f)))
                    .map_err(|_| RuntimeError::new(format!("실수변환 실패: '{}'", s), line)),
                _ => Err(RuntimeError::new("실수변환: 변환 불가 타입", line)),
            }
        }
        "길이" => {
            if args.len() != 1 {
                return Err(RuntimeError::new("길이: 인자 1개 필요", line));
            }
            let v = eval_expr(&args[0], env, line)?;
            match v {
                Value::Str(s) => Ok(Some(Value::Int(s.chars().count() as i64))),
                _ => Err(RuntimeError::new("길이: 문자열 타입 필요", line)),
            }
        }
        _ => Ok(None),
    }
}

fn eval_binary_op(
    op: &BinaryOpKind,
    lv: Value,
    rv: Value,
    line: usize,
) -> Result<Value, RuntimeError> {
    match op {
        BinaryOpKind::Add => match (lv, rv) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + b as f64)),
            (Value::Str(a), Value::Str(b)) => Ok(Value::Str(a + &b)),
            _ => Err(RuntimeError::new("+ 연산: 타입 불일치", line)),
        },
        BinaryOpKind::Sub => match (lv, rv) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 - b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - b as f64)),
            _ => Err(RuntimeError::new("- 연산: 타입 불일치", line)),
        },
        BinaryOpKind::Mul => match (lv, rv) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 * b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * b as f64)),
            _ => Err(RuntimeError::new("* 연산: 타입 불일치", line)),
        },
        BinaryOpKind::Div => match (lv, rv) {
            (Value::Int(a), Value::Int(b)) => {
                if b == 0 {
                    Err(RuntimeError::new("0으로 나눌 수 없습니다", line))
                } else {
                    Ok(Value::Int(a / b))
                }
            }
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 / b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a / b as f64)),
            _ => Err(RuntimeError::new("/ 연산: 타입 불일치", line)),
        },
        BinaryOpKind::Mod => match (lv, rv) {
            (Value::Int(a), Value::Int(b)) => {
                if b == 0 {
                    Err(RuntimeError::new("0으로 나머지 연산 불가", line))
                } else {
                    Ok(Value::Int(a % b))
                }
            }
            _ => Err(RuntimeError::new("% 연산: 정수에만 적용 가능", line)),
        },
        BinaryOpKind::Eq => Ok(Value::Bool(values_equal(&lv, &rv))),
        BinaryOpKind::NotEq => Ok(Value::Bool(!values_equal(&lv, &rv))),
        BinaryOpKind::Lt => compare_values(lv, rv, |a, b| a < b, line),
        BinaryOpKind::Gt => compare_values(lv, rv, |a, b| a > b, line),
        BinaryOpKind::LtEq => compare_values(lv, rv, |a, b| a <= b, line),
        BinaryOpKind::GtEq => compare_values(lv, rv, |a, b| a >= b, line),
        BinaryOpKind::And => match (lv, rv) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
            _ => Err(RuntimeError::new("&& 연산: 불 값에만 적용 가능", line)),
        },
        BinaryOpKind::Or => match (lv, rv) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
            _ => Err(RuntimeError::new("|| 연산: 불 값에만 적용 가능", line)),
        },
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => x == y,
        (Value::Float(x), Value::Float(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Void, Value::Void) => true,
        (Value::Array(x), Value::Array(y)) => {
            let x = x.borrow();
            let y = y.borrow();
            x.len() == y.len() && x.iter().zip(y.iter()).all(|(a, b)| values_equal(a, b))
        }
        _ => false,
    }
}

fn compare_values<F>(lv: Value, rv: Value, cmp: F, line: usize) -> Result<Value, RuntimeError>
where
    F: Fn(f64, f64) -> bool,
{
    match (lv, rv) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(cmp(a as f64, b as f64))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(cmp(a, b))),
        _ => Err(RuntimeError::new(
            "비교 연산: 숫자 타입에만 적용 가능",
            line,
        )),
    }
}

pub fn eval_stmt(stmt: &Stmt, env: &mut Environment) -> Result<Option<Signal>, RuntimeError> {
    let line = stmt.span.line;
    match &stmt.kind {
        StmtKind::VarDecl { name, value, .. } => {
            let val = eval_expr(value, env, line)?;
            env.set(name.clone(), val);
            Ok(None)
        }

        StmtKind::FuncDef {
            name, params, body, ..
        } => {
            let func = Value::Function {
                params: params.clone(),
                body: body.clone(),
            };
            env.set(name.clone(), func);
            Ok(None)
        }

        StmtKind::Return(expr_opt) => {
            let val = match expr_opt {
                Some(expr) => eval_expr(expr, env, line)?,
                None => Value::Void,
            };
            Ok(Some(Signal::Return(val)))
        }

        StmtKind::If {
            cond,
            then_block,
            else_block,
        } => {
            let cond_val = eval_expr(cond, env, line)?;
            match cond_val {
                Value::Bool(true) => eval_block(then_block, env),
                Value::Bool(false) => {
                    if let Some(else_stmts) = else_block {
                        eval_block(else_stmts, env)
                    } else {
                        Ok(None)
                    }
                }
                _ => Err(RuntimeError::new("조건문: 불 값이 필요합니다", line)),
            }
        }

        StmtKind::WhileLoop { cond, body } => {
            loop {
                let cond_val = eval_expr(cond, env, line)?;
                match cond_val {
                    Value::Bool(true) => {}
                    Value::Bool(false) => break,
                    _ => return Err(RuntimeError::new("동안 조건: 불 값이 필요합니다", line)),
                }
                match eval_block(body, env)? {
                    Some(Signal::Break) => break,
                    Some(Signal::Continue) => continue,
                    Some(sig @ Signal::Return(_)) => return Ok(Some(sig)),
                    None => {}
                }
            }
            Ok(None)
        }

        StmtKind::ForLoop {
            init,
            cond,
            step,
            body,
        } => {
            eval_stmt(init, env)?;
            loop {
                let cond_val = eval_expr(cond, env, line)?;
                match cond_val {
                    Value::Bool(true) => {}
                    Value::Bool(false) => break,
                    _ => return Err(RuntimeError::new("반복 조건: 불 값이 필요합니다", line)),
                }
                match eval_block(body, env)? {
                    Some(Signal::Break) => break,
                    Some(Signal::Continue) => {
                        eval_stmt(step, env)?;
                        continue;
                    }
                    Some(sig @ Signal::Return(_)) => return Ok(Some(sig)),
                    None => {}
                }
                eval_stmt(step, env)?;
            }
            Ok(None)
        }

        StmtKind::Break => Ok(Some(Signal::Break)),
        StmtKind::Continue => Ok(Some(Signal::Continue)),

        StmtKind::ExprStmt(expr) => {
            eval_expr(expr, env, line)?;
            Ok(None)
        }

        StmtKind::StructDef { name, .. } => {
            env.set(name.clone(), Value::Str(format!("<구조체 {}>", name)));
            Ok(None)
        }

        StmtKind::TryCatch {
            try_block,
            error_name,
            catch_block,
        } => match eval_block(try_block, env) {
            Ok(sig) => Ok(sig),
            Err(e) => {
                env.set(error_name.clone(), Value::Str(e.message.clone()));
                eval_block(catch_block, env)
            }
        },

        StmtKind::Import(path) => {
            let source = std::fs::read_to_string(path)
                .map_err(|e| RuntimeError::new(format!("가져오기 실패 '{}': {}", path, e), line))?;
            let tokens = crate::lexer::tokenize(&source);
            let program = crate::parser::parse(tokens).map_err(|e| {
                RuntimeError::new(format!("'{}' 파싱 오류: {}", path, e.message), line)
            })?;
            eval_block(&program.stmts, env)?;
            Ok(None)
        }

        StmtKind::Match { expr, arms } => {
            let val = eval_expr(expr, env, line)?;
            for arm in arms {
                if pattern_matches(&arm.pattern, &val, env) {
                    return eval_block(&arm.body, env);
                }
            }
            Ok(None)
        }

        StmtKind::ImplBlock {
            struct_name,
            methods,
        } => {
            for method_stmt in methods {
                if let StmtKind::FuncDef {
                    name,
                    params,
                    return_type: _,
                    body,
                } = &method_stmt.kind
                {
                    let key = format!("{}::{}", struct_name, name);
                    let func = Value::Function {
                        params: params.clone(),
                        body: body.clone(),
                    };
                    env.set(key, func);
                }
            }
            Ok(None)
        }

        StmtKind::EnumDef { name, variants } => {
            for (i, variant) in variants.iter().enumerate() {
                let key = format!("{}::{}", name, variant);
                env.set(key, Value::Int(i as i64));
            }
            Ok(None)
        }

        StmtKind::ForIn {
            var_name,
            iterable,
            body,
        } => {
            let iter_val = eval_expr(iterable, env, line)?;
            match iter_val {
                Value::Array(arr) => {
                    let items = arr.borrow().clone();
                    for item in items {
                        env.set(var_name.clone(), item);
                        match eval_block(body, env)? {
                            Some(Signal::Break) => break,
                            Some(Signal::Continue) => continue,
                            Some(sig @ Signal::Return(_)) => return Ok(Some(sig)),
                            None => {}
                        }
                    }
                    Ok(None)
                }
                Value::Str(s) => {
                    for ch in s.chars() {
                        env.set(var_name.clone(), Value::Str(ch.to_string()));
                        match eval_block(body, env)? {
                            Some(Signal::Break) => break,
                            Some(Signal::Continue) => continue,
                            Some(sig @ Signal::Return(_)) => return Ok(Some(sig)),
                            None => {}
                        }
                    }
                    Ok(None)
                }
                _ => Err(RuntimeError::new(
                    "반복 안에서: 배열 또는 문자열 필요",
                    line,
                )),
            }
        }
    }
}

fn pattern_matches(pattern: &Pattern, value: &Value, env: &mut Environment) -> bool {
    match (pattern, value) {
        (Pattern::Wildcard, _) => true,
        (Pattern::IntLiteral(n), Value::Int(v)) => n == v,
        (Pattern::FloatLiteral(f), Value::Float(v)) => (f - v).abs() < f64::EPSILON,
        (Pattern::StringLiteral(s), Value::Str(v)) => s == v,
        (Pattern::BoolLiteral(b), Value::Bool(v)) => b == v,
        (Pattern::Identifier(name), val) => {
            env.set(name.clone(), val.clone());
            true
        }
        (Pattern::ArrayPattern(pats), Value::Array(arr)) => {
            let arr = arr.borrow();
            if pats.len() != arr.len() {
                return false;
            }
            pats.iter()
                .zip(arr.iter())
                .all(|(p, v)| pattern_matches(p, v, env))
        }
        _ => false,
    }
}

pub fn eval_block(stmts: &[Stmt], env: &mut Environment) -> Result<Option<Signal>, RuntimeError> {
    for stmt in stmts {
        if let Some(sig) = eval_stmt(stmt, env)? {
            return Ok(Some(sig));
        }
    }
    Ok(None)
}

pub fn interpret(program: Program) -> Result<(), RuntimeError> {
    let mut env = Environment::new();
    eval_block(&program.stmts, &mut env)?;
    Ok(())
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

    #[test]
    fn test_eval_arithmetic() {
        let mut env = Environment::new();
        let expr = Expr::BinaryOp {
            op: BinaryOpKind::Add,
            left: Box::new(Expr::IntLiteral(3)),
            right: Box::new(Expr::BinaryOp {
                op: BinaryOpKind::Mul,
                left: Box::new(Expr::IntLiteral(5)),
                right: Box::new(Expr::IntLiteral(2)),
            }),
        };
        let result = eval_expr(&expr, &mut env, 0).unwrap();
        assert!(matches!(result, Value::Int(13)));
    }

    #[test]
    fn test_eval_var_decl() {
        let mut env = Environment::new();
        let stmt = Stmt::unspanned(StmtKind::VarDecl {
            name: "나이".to_string(),
            ty: None,
            value: Expr::IntLiteral(20),
            mutable: true,
        });
        eval_stmt(&stmt, &mut env).unwrap();
        assert!(matches!(env.get("나이"), Some(Value::Int(20))));
    }

    #[test]
    fn test_eval_if_stmt() {
        let mut env = Environment::new();
        let stmt = Stmt::unspanned(StmtKind::If {
            cond: Expr::BoolLiteral(true),
            then_block: vec![Stmt::unspanned(StmtKind::VarDecl {
                name: "x".to_string(),
                ty: None,
                value: Expr::IntLiteral(1),
                mutable: false,
            })],
            else_block: None,
        });
        eval_stmt(&stmt, &mut env).unwrap();
        assert!(matches!(env.get("x"), Some(Value::Int(1))));
    }

    #[test]
    fn test_eval_fibonacci() {
        let fib_body = vec![
            Stmt::unspanned(StmtKind::If {
                cond: Expr::BinaryOp {
                    op: BinaryOpKind::LtEq,
                    left: Box::new(Expr::Identifier("n".to_string())),
                    right: Box::new(Expr::IntLiteral(1)),
                },
                then_block: vec![Stmt::unspanned(StmtKind::Return(Some(Expr::Identifier(
                    "n".to_string(),
                ))))],
                else_block: None,
            }),
            Stmt::unspanned(StmtKind::Return(Some(Expr::BinaryOp {
                op: BinaryOpKind::Add,
                left: Box::new(Expr::Call {
                    name: "피보나치".to_string(),
                    args: vec![Expr::BinaryOp {
                        op: BinaryOpKind::Sub,
                        left: Box::new(Expr::Identifier("n".to_string())),
                        right: Box::new(Expr::IntLiteral(1)),
                    }],
                }),
                right: Box::new(Expr::Call {
                    name: "피보나치".to_string(),
                    args: vec![Expr::BinaryOp {
                        op: BinaryOpKind::Sub,
                        left: Box::new(Expr::Identifier("n".to_string())),
                        right: Box::new(Expr::IntLiteral(2)),
                    }],
                }),
            }))),
        ];

        let program = Program::new(vec![
            Stmt::unspanned(StmtKind::FuncDef {
                name: "피보나치".to_string(),
                params: vec![("n".to_string(), Type::정수)],
                return_type: Some(Type::정수),
                body: fib_body,
            }),
            Stmt::unspanned(StmtKind::VarDecl {
                name: "결과".to_string(),
                ty: None,
                value: Expr::Call {
                    name: "피보나치".to_string(),
                    args: vec![Expr::IntLiteral(10)],
                },
                mutable: false,
            }),
        ]);

        let mut env = Environment::new();
        eval_block(&program.stmts, &mut env).unwrap();
        assert!(matches!(env.get("결과"), Some(Value::Int(55))));
    }

    #[test]
    fn test_출력() {
        let program = Program::new(vec![Stmt::unspanned(StmtKind::ExprStmt(Expr::Call {
            name: "출력".to_string(),
            args: vec![Expr::StringLiteral("안녕".to_string())],
        }))]);
        let result = interpret(program);
        assert!(result.is_ok());
    }
}
