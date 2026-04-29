use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{self, BufRead};
use std::rc::Rc;

use crate::ast::{
    BinaryOpKind, Expr, ExprVisitor, Pattern, Program, Stmt, StmtKind, StmtVisitor, UnaryOpKind,
};

use super::builtins::{eval_builtin_io, eval_builtin_math, eval_builtin_stdlib};
use super::env::{EnvRef, Environment, RuntimeError};
use super::output_line;
use super::value::Value;

pub enum Signal {
    Return(Value),
    Break,
    Continue,
}

pub struct Evaluator<'a> {
    pub env: &'a EnvRef,
    pub line: usize,
}

impl<'a> Evaluator<'a> {
    pub fn new(env: &'a EnvRef, line: usize) -> Self {
        Self { env, line }
    }
}

pub fn eval_expr(expr: &Expr, env: &EnvRef, line: usize) -> Result<Value, RuntimeError> {
    Evaluator::new(env, line).visit_expr(expr)
}

pub fn eval_stmt(stmt: &Stmt, env: &EnvRef) -> Result<Option<Signal>, RuntimeError> {
    Evaluator::new(env, stmt.span.line).visit_stmt(stmt)
}

impl<'a> ExprVisitor<Result<Value, RuntimeError>> for Evaluator<'a> {
    fn visit_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        let env = self.env;
        let line = self.line;
        match expr {
            Expr::IntLiteral(n) => Ok(Value::Int(*n)),
            Expr::FloatLiteral(f) => Ok(Value::Float(*f)),
            Expr::StringLiteral(s) => Ok(Value::Str(s.clone())),
            Expr::BoolLiteral(b) => Ok(Value::Bool(*b)),
            Expr::NullLiteral => Ok(Value::Void),

            Expr::Identifier(name) => env
                .borrow()
                .get(name)
                .ok_or_else(|| RuntimeError::new(format!("정의되지 않은 변수: {}", name), line)),

            Expr::Assign { name, value } => {
                if env.borrow().is_const(name) {
                    return Err(RuntimeError::new(
                        format!("상수 '{}'는 재할당할 수 없습니다", name),
                        line,
                    ));
                }
                let val = self.visit_expr(value)?;
                let updated = env.borrow_mut().update(name, val.clone());
                if !updated {
                    env.borrow_mut().set(name.clone(), val.clone());
                }
                Ok(val)
            }

            Expr::BinaryOp { op, left, right } => {
                let lv = self.visit_expr(left)?;
                let rv = self.visit_expr(right)?;
                eval_binary_op(op, lv, rv, line)
            }

            Expr::UnaryOp { op, expr } => {
                let val = self.visit_expr(expr)?;
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
                        let v = self.visit_expr(arg)?;
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
                        let key = self.visit_expr(&args[i])?;
                        let val = self.visit_expr(&args[i + 1])?;
                        pairs.push((key, val));
                        i += 2;
                    }
                    return Ok(Value::Map(Rc::new(RefCell::new(pairs))));
                }

                if let Some(result) = eval_builtin_io(name, args, env, line)? {
                    return Ok(result);
                }

                let func_val = env.borrow().get(name).ok_or_else(|| {
                    RuntimeError::new(format!("정의되지 않은 함수: {}", name), line)
                })?;

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
                            arg_vals.push(self.visit_expr(arg)?);
                        }

                        let func_env = Environment::new_enclosed_ref(env.clone());
                        for ((param_name, _ty), val) in params.iter().zip(arg_vals) {
                            func_env.borrow_mut().set(param_name.clone(), val);
                        }

                        match eval_block(&body, &func_env) {
                            Ok(Some(Signal::Return(v))) => Ok(v),
                            Ok(_) => Ok(Value::Void),
                            Err(e) => {
                                Err(e.with_frame(format!("  함수 '{}' ({}번째 줄)", name, line)))
                            }
                        }
                    }
                    Value::Closure {
                        params,
                        body,
                        env: captured_env,
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
                            arg_vals.push(self.visit_expr(arg)?);
                        }
                        let closure_env = Environment::new_enclosed_ref(captured_env);
                        for ((param_name, _), val) in params.iter().zip(arg_vals) {
                            closure_env.borrow_mut().set(param_name.clone(), val);
                        }
                        match eval_block(&body, &closure_env)? {
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
                    vals.push(self.visit_expr(e)?);
                }
                Ok(Value::Array(Rc::new(RefCell::new(vals))))
            }

            Expr::Index { object, index } => {
                let obj = self.visit_expr(object)?;
                let idx = self.visit_expr(index)?;
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
                let obj = self.visit_expr(object)?;
                let idx = self.visit_expr(index)?;
                let val = self.visit_expr(value)?;
                match (obj, idx) {
                    (Value::Array(arr), Value::Int(i)) => {
                        let mut arr = arr.borrow_mut();
                        let len = arr.len() as i64;
                        let i = if i < 0 { len + i } else { i };
                        if i < 0 || i >= len {
                            return Err(RuntimeError::new(
                                format!("인덱스 범위 초과: {}", i),
                                line,
                            ));
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
                let obj = self.visit_expr(object)?;
                eval_method(obj, method, args, env, line)
            }

            Expr::FieldAccess { object, field } => {
                let obj = self.visit_expr(object)?;
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
                let obj = self.visit_expr(object)?;
                let val = self.visit_expr(value)?;
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
                    map.insert(fname.clone(), self.visit_expr(fexpr)?);
                }
                Ok(Value::Struct {
                    name: name.clone(),
                    fields: Rc::new(RefCell::new(map)),
                })
            }

            Expr::Lambda { params, body } => Ok(Value::Closure {
                params: params.clone(),
                body: body.clone(),
                env: env.clone(),
            }),

            Expr::Range { start, end } => {
                let s = match self.visit_expr(start)? {
                    Value::Int(n) => n,
                    _ => return Err(RuntimeError::new("범위: 정수 필요", line)),
                };
                let e = match self.visit_expr(end)? {
                    Value::Int(n) => n,
                    _ => return Err(RuntimeError::new("범위: 정수 필요", line)),
                };
                let vals: Vec<Value> = (s..e).map(Value::Int).collect();
                Ok(Value::Array(Rc::new(RefCell::new(vals))))
            }

            Expr::TupleLiteral(elems) => {
                let mut vals = Vec::new();
                for e in elems {
                    vals.push(self.visit_expr(e)?);
                }
                Ok(Value::Tuple(vals))
            }

            Expr::TupleIndex { object, index } => {
                let obj = self.visit_expr(object)?;
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
                    let key = self.visit_expr(k)?;
                    let val = self.visit_expr(v)?;
                    pairs.push((key, val));
                }
                Ok(Value::Map(Rc::new(RefCell::new(pairs))))
            }
        }
    }
}

fn eval_method(
    obj: Value,
    method: &str,
    args: &[Expr],
    env: &EnvRef,
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
            let func_val = env.borrow().get(&method_key);
            if let Some(func_val) = func_val {
                match func_val {
                    Value::Function { params, body } => {
                        let method_env = Environment::new_enclosed_ref(env.clone());
                        method_env.borrow_mut().set(
                            "자신".to_string(),
                            Value::Struct {
                                name: struct_name.clone(),
                                fields: fields.clone(),
                            },
                        );
                        let non_self_params: Vec<_> =
                            params.iter().filter(|(n, _)| n != "자신").collect();
                        for ((pname, _), val) in non_self_params.iter().zip(arg_vals) {
                            method_env.borrow_mut().set(pname.clone(), val);
                        }
                        match eval_block(&body, &method_env)? {
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

impl<'a> StmtVisitor<Result<Option<Signal>, RuntimeError>> for Evaluator<'a> {
    fn visit_stmt(&mut self, stmt: &Stmt) -> Result<Option<Signal>, RuntimeError> {
        self.line = stmt.span.line;
        let env = self.env;
        let line = self.line;
        match &stmt.kind {
            StmtKind::VarDecl {
                name,
                value,
                mutable,
                ..
            } => {
                let val = self.visit_expr(value)?;
                if *mutable {
                    env.borrow_mut().set(name.clone(), val);
                } else {
                    env.borrow_mut().set_const(name.clone(), val);
                }
                Ok(None)
            }

            StmtKind::FuncDef {
                name, params, body, ..
            } => {
                let func = Value::Function {
                    params: params.clone(),
                    body: body.clone(),
                };
                env.borrow_mut().set(name.clone(), func);
                Ok(None)
            }

            StmtKind::Return(expr_opt) => {
                let val = match expr_opt {
                    Some(expr) => self.visit_expr(expr)?,
                    None => Value::Void,
                };
                Ok(Some(Signal::Return(val)))
            }

            StmtKind::If {
                cond,
                then_block,
                else_block,
            } => {
                let cond_val = self.visit_expr(cond)?;
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
                    let cond_val = self.visit_expr(cond)?;
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
                self.visit_stmt(init)?;
                loop {
                    let cond_val = self.visit_expr(cond)?;
                    match cond_val {
                        Value::Bool(true) => {}
                        Value::Bool(false) => break,
                        _ => return Err(RuntimeError::new("반복 조건: 불 값이 필요합니다", line)),
                    }
                    match eval_block(body, env)? {
                        Some(Signal::Break) => break,
                        Some(Signal::Continue) => {
                            self.visit_stmt(step)?;
                            continue;
                        }
                        Some(sig @ Signal::Return(_)) => return Ok(Some(sig)),
                        None => {}
                    }
                    self.visit_stmt(step)?;
                }
                Ok(None)
            }

            StmtKind::Break => Ok(Some(Signal::Break)),
            StmtKind::Continue => Ok(Some(Signal::Continue)),

            StmtKind::ExprStmt(expr) => {
                self.visit_expr(expr)?;
                Ok(None)
            }

            StmtKind::StructDef { name, .. } => {
                env.borrow_mut()
                    .set(name.clone(), Value::Str(format!("<구조체 {}>", name)));
                Ok(None)
            }

            StmtKind::TryCatch {
                try_block,
                error_name,
                catch_block,
            } => match eval_block(try_block, env) {
                Ok(sig) => Ok(sig),
                Err(e) => {
                    env.borrow_mut()
                        .set(error_name.clone(), Value::Str(e.message.clone()));
                    eval_block(catch_block, env)
                }
            },

            StmtKind::Import(path) => {
                let resolved_path = std::fs::canonicalize(path)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| path.clone());

                if env.borrow().has_imported_path(&resolved_path) {
                    return Ok(None);
                }

                env.borrow_mut().mark_imported_path(&resolved_path);

                let import_result = (|| -> Result<(), RuntimeError> {
                    let source = std::fs::read_to_string(&resolved_path).map_err(|e| {
                        RuntimeError::new(format!("포함 실패 '{}': {}", path, e), line)
                    })?;
                    let tokens = crate::lexer::tokenize(&source);
                    let program = crate::parser::parse(tokens).map_err(|e| {
                        RuntimeError::new(format!("'{}' 파싱 오류: {}", path, e.message), line)
                    })?;
                    eval_block(&program.stmts, env)?;
                    Ok(())
                })();

                if let Err(err) = import_result {
                    env.borrow_mut().unmark_imported_path(&resolved_path);
                    return Err(err);
                }

                Ok(None)
            }

            StmtKind::Match { expr, arms } => {
                let val = self.visit_expr(expr)?;
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
                        env.borrow_mut().set(key, func);
                    }
                }
                Ok(None)
            }

            StmtKind::EnumDef { name, variants } => {
                for (i, variant) in variants.iter().enumerate() {
                    let key = format!("{}::{}", name, variant);
                    env.borrow_mut().set(key, Value::Int(i as i64));
                }
                Ok(None)
            }

            StmtKind::ForIn {
                var_name,
                iterable,
                body,
            } => {
                let iter_val = self.visit_expr(iterable)?;
                match iter_val {
                    Value::Array(arr) => {
                        let items = arr.borrow().clone();
                        for item in items {
                            env.borrow_mut().set(var_name.clone(), item);
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
                            env.borrow_mut()
                                .set(var_name.clone(), Value::Str(ch.to_string()));
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
}

fn pattern_matches(pattern: &Pattern, value: &Value, env: &EnvRef) -> bool {
    match (pattern, value) {
        (Pattern::Wildcard, _) => true,
        (Pattern::IntLiteral(n), Value::Int(v)) => n == v,
        (Pattern::FloatLiteral(f), Value::Float(v)) => (f - v).abs() < f64::EPSILON,
        (Pattern::StringLiteral(s), Value::Str(v)) => s == v,
        (Pattern::BoolLiteral(b), Value::Bool(v)) => b == v,
        (Pattern::Identifier(name), val) => {
            env.borrow_mut().set(name.clone(), val.clone());
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

pub fn eval_block(stmts: &[Stmt], env: &EnvRef) -> Result<Option<Signal>, RuntimeError> {
    for stmt in stmts {
        if let Some(sig) = eval_stmt(stmt, env)? {
            return Ok(Some(sig));
        }
    }
    Ok(None)
}

pub fn interpret(program: Program) -> Result<(), RuntimeError> {
    let env = Environment::new_ref();
    eval_block(&program.stmts, &env)?;
    Ok(())
}
