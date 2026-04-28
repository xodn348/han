use std::cell::RefCell;
use std::rc::Rc;

use crate::ast::Expr;

use super::env::{EnvRef, RuntimeError};
use super::evaluator::eval_expr;
use super::value::{json_to_value, value_to_json, Value};

pub(crate) fn eval_builtin_stdlib(
    name: &str,
    args: &[Expr],
    env: &EnvRef,
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
        "HTTP_포함" => {
            #[cfg(feature = "native")]
            {
                if args.len() != 1 {
                    return Err(RuntimeError::new("HTTP_포함: URL 인자 필요", line));
                }
                let url = match eval_expr(&args[0], env, line)? {
                    Value::Str(s) => s,
                    _ => return Err(RuntimeError::new("HTTP_포함: 문자열 URL 필요", line)),
                };
                let body = reqwest::blocking::get(&url)
                    .map_err(|e| RuntimeError::new(format!("HTTP 오류: {}", e), line))?
                    .text()
                    .map_err(|e| RuntimeError::new(format!("HTTP 응답 읽기 오류: {}", e), line))?;
                Ok(Some(Value::Str(body)))
            }
            #[cfg(not(feature = "native"))]
            {
                let _ = (args, env);
                Err(RuntimeError::new(
                    "HTTP_포함: 플레이그라운드에서 미지원",
                    line,
                ))
            }
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
                Ok(Some(Value::Str(resp)))
            }
            #[cfg(not(feature = "native"))]
            {
                let _ = (args, env);
                Err(RuntimeError::new(
                    "HTTP_보내기: 플레이그라운드에서 미지원",
                    line,
                ))
            }
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
                Ok(Some(Value::Str(
                    String::from_utf8_lossy(&output.stdout).to_string(),
                )))
            }
            #[cfg(not(feature = "native"))]
            {
                let _ = (args, env);
                Err(RuntimeError::new("실행: 플레이그라운드에서 미지원", line))
            }
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

pub(crate) fn eval_builtin_io(
    name: &str,
    args: &[Expr],
    env: &EnvRef,
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
                let snapshot = env.borrow().snapshot();
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

pub(crate) fn eval_builtin_math(
    name: &str,
    args: &[Expr],
    env: &EnvRef,
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
        "사인" => {
            if args.len() != 1 {
                return Err(RuntimeError::new("사인: 인자 1개 필요", line));
            }
            let v = eval_expr(&args[0], env, line)?;
            let x = match v {
                Value::Int(n) => n as f64,
                Value::Float(f) => f,
                _ => return Err(RuntimeError::new("사인: 숫자 필요", line)),
            };
            Ok(Some(Value::Float(x.sin())))
        }
        "코사인" => {
            if args.len() != 1 {
                return Err(RuntimeError::new("코사인: 인자 1개 필요", line));
            }
            let v = eval_expr(&args[0], env, line)?;
            let x = match v {
                Value::Int(n) => n as f64,
                Value::Float(f) => f,
                _ => return Err(RuntimeError::new("코사인: 숫자 필요", line)),
            };
            Ok(Some(Value::Float(x.cos())))
        }
        "탄젠트" => {
            if args.len() != 1 {
                return Err(RuntimeError::new("탄젠트: 인자 1개 필요", line));
            }
            let v = eval_expr(&args[0], env, line)?;
            let x = match v {
                Value::Int(n) => n as f64,
                Value::Float(f) => f,
                _ => return Err(RuntimeError::new("탄젠트: 숫자 필요", line)),
            };
            Ok(Some(Value::Float(x.tan())))
        }
        "로그" => {
            if args.len() != 1 {
                return Err(RuntimeError::new("로그: 인자 1개 필요 (자연로그)", line));
            }
            let v = eval_expr(&args[0], env, line)?;
            let x = match v {
                Value::Int(n) => n as f64,
                Value::Float(f) => f,
                _ => return Err(RuntimeError::new("로그: 숫자 필요", line)),
            };
            Ok(Some(Value::Float(x.ln())))
        }
        "로그10" => {
            if args.len() != 1 {
                return Err(RuntimeError::new("로그10: 인자 1개 필요", line));
            }
            let v = eval_expr(&args[0], env, line)?;
            let x = match v {
                Value::Int(n) => n as f64,
                Value::Float(f) => f,
                _ => return Err(RuntimeError::new("로그10: 숫자 필요", line)),
            };
            Ok(Some(Value::Float(x.log10())))
        }
        "지수" => {
            if args.len() != 1 {
                return Err(RuntimeError::new("지수: 인자 1개 필요 (e^x)", line));
            }
            let v = eval_expr(&args[0], env, line)?;
            let x = match v {
                Value::Int(n) => n as f64,
                Value::Float(f) => f,
                _ => return Err(RuntimeError::new("지수: 숫자 필요", line)),
            };
            Ok(Some(Value::Float(x.exp())))
        }
        "올림" => {
            if args.len() != 1 {
                return Err(RuntimeError::new("올림: 인자 1개 필요", line));
            }
            let v = eval_expr(&args[0], env, line)?;
            let x = match v {
                Value::Int(n) => n as f64,
                Value::Float(f) => f,
                _ => return Err(RuntimeError::new("올림: 숫자 필요", line)),
            };
            Ok(Some(Value::Float(x.ceil())))
        }
        "내림" => {
            if args.len() != 1 {
                return Err(RuntimeError::new("내림: 인자 1개 필요", line));
            }
            let v = eval_expr(&args[0], env, line)?;
            let x = match v {
                Value::Int(n) => n as f64,
                Value::Float(f) => f,
                _ => return Err(RuntimeError::new("내림: 숫자 필요", line)),
            };
            Ok(Some(Value::Float(x.floor())))
        }
        "반올림" => {
            if args.len() != 1 {
                return Err(RuntimeError::new("반올림: 인자 1개 필요", line));
            }
            let v = eval_expr(&args[0], env, line)?;
            let x = match v {
                Value::Int(n) => n as f64,
                Value::Float(f) => f,
                _ => return Err(RuntimeError::new("반올림: 숫자 필요", line)),
            };
            Ok(Some(Value::Float(x.round())))
        }
        "최대" => {
            if args.len() != 2 {
                return Err(RuntimeError::new("최대: 인자 2개 필요", line));
            }
            let a = eval_expr(&args[0], env, line)?;
            let b = eval_expr(&args[1], env, line)?;
            let av = match &a {
                Value::Int(n) => *n as f64,
                Value::Float(f) => *f,
                _ => return Err(RuntimeError::new("최대: 숫자 필요", line)),
            };
            let bv = match &b {
                Value::Int(n) => *n as f64,
                Value::Float(f) => *f,
                _ => return Err(RuntimeError::new("최대: 숫자 필요", line)),
            };
            Ok(Some(Value::Float(av.max(bv))))
        }
        "최소" => {
            if args.len() != 2 {
                return Err(RuntimeError::new("최소: 인자 2개 필요", line));
            }
            let a = eval_expr(&args[0], env, line)?;
            let b = eval_expr(&args[1], env, line)?;
            let av = match &a {
                Value::Int(n) => *n as f64,
                Value::Float(f) => *f,
                _ => return Err(RuntimeError::new("최소: 숫자 필요", line)),
            };
            let bv = match &b {
                Value::Int(n) => *n as f64,
                Value::Float(f) => *f,
                _ => return Err(RuntimeError::new("최소: 숫자 필요", line)),
            };
            Ok(Some(Value::Float(av.min(bv))))
        }
        "난수" => {
            if args.is_empty() {
                let r: f64 = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .subsec_nanos() as f64
                    / 1_000_000_000.0;
                Ok(Some(Value::Float(r)))
            } else if args.len() == 2 {
                let a = match eval_expr(&args[0], env, line)? {
                    Value::Int(n) => n,
                    _ => return Err(RuntimeError::new("난수: 정수 필요", line)),
                };
                let b = match eval_expr(&args[1], env, line)? {
                    Value::Int(n) => n,
                    _ => return Err(RuntimeError::new("난수: 정수 필요", line)),
                };
                let nanos = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .subsec_nanos() as i64;
                Ok(Some(Value::Int(a + (nanos.abs() % (b - a + 1)))))
            } else {
                Err(RuntimeError::new(
                    "난수: 인자 0개 또는 2개 필요 (최소, 최대)",
                    line,
                ))
            }
        }
        "파이" => {
            if !args.is_empty() {
                return Err(RuntimeError::new("파이: 인자 없음", line));
            }
            Ok(Some(Value::Float(std::f64::consts::PI)))
        }
        "자연상수" => {
            if !args.is_empty() {
                return Err(RuntimeError::new("자연상수: 인자 없음", line));
            }
            Ok(Some(Value::Float(std::f64::consts::E)))
        }
        #[cfg(feature = "python")]
        "파이썬" => {
            if args.len() != 1 {
                return Err(RuntimeError::new(
                    "파이썬: 인자 1개 필요 (Python 코드 문자열)",
                    line,
                ));
            }
            let code = match eval_expr(&args[0], env, line)? {
                Value::Str(s) => s,
                _ => return Err(RuntimeError::new("파이썬: 문자열 필요", line)),
            };
            match crate::python_interop::run_python(&code) {
                Ok(result) => Ok(Some(Value::Str(result))),
                Err(e) => Err(RuntimeError::new(format!("파이썬 에러: {}", e), line)),
            }
        }
        #[cfg(feature = "python")]
        "파이썬_값" => {
            if args.len() != 1 {
                return Err(RuntimeError::new("파이썬_값: 인자 1개 필요", line));
            }
            let code = match eval_expr(&args[0], env, line)? {
                Value::Str(s) => s,
                _ => return Err(RuntimeError::new("파이썬_값: 문자열 필요", line)),
            };
            match crate::python_interop::eval_python(&code) {
                Ok(val) => Ok(Some(val)),
                Err(e) => Err(RuntimeError::new(format!("파이썬 에러: {}", e), line)),
            }
        }
        "행렬곱" => {
            if args.len() != 2 {
                return Err(RuntimeError::new("행렬곱: 인자 2개 필요 (A, B)", line));
            }
            let a = eval_expr(&args[0], env, line)?;
            let b = eval_expr(&args[1], env, line)?;
            match (a, b) {
                (Value::Array(a_rows), Value::Array(b_rows)) => {
                    let a_rows = a_rows.borrow();
                    let b_rows = b_rows.borrow();
                    let m = a_rows.len();
                    if m == 0 {
                        return Ok(Some(Value::Array(Rc::new(RefCell::new(vec![])))));
                    }
                    let a_cols = match &a_rows[0] {
                        Value::Array(r) => r.borrow().len(),
                        _ => return Err(RuntimeError::new("행렬곱: 2차원 배열 필요", line)),
                    };
                    let b_cols = match &b_rows[0] {
                        Value::Array(r) => r.borrow().len(),
                        _ => return Err(RuntimeError::new("행렬곱: 2차원 배열 필요", line)),
                    };
                    let mut result = Vec::with_capacity(m);
                    for i in 0..m {
                        let a_row = match &a_rows[i] {
                            Value::Array(r) => r.borrow().clone(),
                            _ => return Err(RuntimeError::new("행렬곱: 2차원 배열 필요", line)),
                        };
                        let mut row = Vec::with_capacity(b_cols);
                        for j in 0..b_cols {
                            let mut sum = 0.0_f64;
                            for k in 0..a_cols {
                                let av = match &a_row[k] {
                                    Value::Int(n) => *n as f64,
                                    Value::Float(f) => *f,
                                    _ => {
                                        return Err(RuntimeError::new(
                                            "행렬곱: 숫자 타입 필요",
                                            line,
                                        ));
                                    }
                                };
                                let bv = match &b_rows[k] {
                                    Value::Array(br) => {
                                        let br = br.borrow();
                                        match &br[j] {
                                            Value::Int(n) => *n as f64,
                                            Value::Float(f) => *f,
                                            _ => {
                                                return Err(RuntimeError::new(
                                                    "행렬곱: 숫자 타입 필요",
                                                    line,
                                                ));
                                            }
                                        }
                                    }
                                    _ => {
                                        return Err(RuntimeError::new(
                                            "행렬곱: 2차원 배열 필요",
                                            line,
                                        ));
                                    }
                                };
                                sum += av * bv;
                            }
                            row.push(Value::Float(sum));
                        }
                        result.push(Value::Array(Rc::new(RefCell::new(row))));
                    }
                    Ok(Some(Value::Array(Rc::new(RefCell::new(result)))))
                }
                _ => Err(RuntimeError::new("행렬곱: 2차원 배열 필요", line)),
            }
        }
        "전치" => {
            if args.len() != 1 {
                return Err(RuntimeError::new("전치: 인자 1개 필요", line));
            }
            let v = eval_expr(&args[0], env, line)?;
            match v {
                Value::Array(rows) => {
                    let rows = rows.borrow();
                    if rows.is_empty() {
                        return Ok(Some(Value::Array(Rc::new(RefCell::new(vec![])))));
                    }
                    let m = rows.len();
                    let n = match &rows[0] {
                        Value::Array(r) => r.borrow().len(),
                        _ => return Err(RuntimeError::new("전치: 2차원 배열 필요", line)),
                    };
                    let mut result = Vec::with_capacity(n);
                    for j in 0..n {
                        let mut col = Vec::with_capacity(m);
                        for i in 0..m {
                            match &rows[i] {
                                Value::Array(r) => {
                                    let r = r.borrow();
                                    col.push(r[j].clone());
                                }
                                _ => return Err(RuntimeError::new("전치: 2차원 배열 필요", line)),
                            }
                        }
                        result.push(Value::Array(Rc::new(RefCell::new(col))));
                    }
                    Ok(Some(Value::Array(Rc::new(RefCell::new(result)))))
                }
                _ => Err(RuntimeError::new("전치: 배열 타입 필요", line)),
            }
        }
        "스칼라곱" => {
            if args.len() != 2 {
                return Err(RuntimeError::new(
                    "스칼라곱: 인자 2개 필요 (행렬, 스칼라)",
                    line,
                ));
            }
            let mat = eval_expr(&args[0], env, line)?;
            let scalar = eval_expr(&args[1], env, line)?;
            let s = match &scalar {
                Value::Int(n) => *n as f64,
                Value::Float(f) => *f,
                _ => return Err(RuntimeError::new("스칼라곱: 두 번째 인자는 숫자", line)),
            };
            match mat {
                Value::Array(rows) => {
                    let rows = rows.borrow();
                    let mut result = Vec::with_capacity(rows.len());
                    for row in rows.iter() {
                        match row {
                            Value::Array(r) => {
                                let r = r.borrow();
                                let new_row: Vec<Value> = r
                                    .iter()
                                    .map(|v| {
                                        let val = match v {
                                            Value::Int(n) => *n as f64,
                                            Value::Float(f) => *f,
                                            _ => 0.0,
                                        };
                                        Value::Float(val * s)
                                    })
                                    .collect();
                                result.push(Value::Array(Rc::new(RefCell::new(new_row))));
                            }
                            _ => return Err(RuntimeError::new("스칼라곱: 2차원 배열 필요", line)),
                        }
                    }
                    Ok(Some(Value::Array(Rc::new(RefCell::new(result)))))
                }
                _ => Err(RuntimeError::new(
                    "스칼라곱: 첫 번째 인자는 2차원 배열",
                    line,
                )),
            }
        }
        "내적" => {
            if args.len() != 2 {
                return Err(RuntimeError::new("내적: 인자 2개 필요", line));
            }
            let a = eval_expr(&args[0], env, line)?;
            let b = eval_expr(&args[1], env, line)?;
            match (a, b) {
                (Value::Array(av), Value::Array(bv)) => {
                    let av = av.borrow();
                    let bv = bv.borrow();
                    if av.len() != bv.len() {
                        return Err(RuntimeError::new("내적: 벡터 길이가 같아야 합니다", line));
                    }
                    let mut sum = 0.0_f64;
                    for i in 0..av.len() {
                        let a_val = match &av[i] {
                            Value::Int(n) => *n as f64,
                            Value::Float(f) => *f,
                            _ => return Err(RuntimeError::new("내적: 숫자 타입 필요", line)),
                        };
                        let b_val = match &bv[i] {
                            Value::Int(n) => *n as f64,
                            Value::Float(f) => *f,
                            _ => return Err(RuntimeError::new("내적: 숫자 타입 필요", line)),
                        };
                        sum += a_val * b_val;
                    }
                    Ok(Some(Value::Float(sum)))
                }
                _ => Err(RuntimeError::new("내적: 1차원 배열 2개 필요", line)),
            }
        }
        "외적" => {
            if args.len() != 2 {
                return Err(RuntimeError::new("외적: 인자 2개 필요 (3차원 벡터)", line));
            }
            let a = eval_expr(&args[0], env, line)?;
            let b = eval_expr(&args[1], env, line)?;
            match (a, b) {
                (Value::Array(av), Value::Array(bv)) => {
                    let av = av.borrow();
                    let bv = bv.borrow();
                    if av.len() != 3 || bv.len() != 3 {
                        return Err(RuntimeError::new("외적: 3차원 벡터만 지원", line));
                    }
                    let g = |arr: &[Value], i: usize| -> Result<f64, RuntimeError> {
                        match &arr[i] {
                            Value::Int(n) => Ok(*n as f64),
                            Value::Float(f) => Ok(*f),
                            _ => Err(RuntimeError::new("외적: 숫자 타입 필요", line)),
                        }
                    };
                    let (a0, a1, a2) = (g(&av, 0)?, g(&av, 1)?, g(&av, 2)?);
                    let (b0, b1, b2) = (g(&bv, 0)?, g(&bv, 1)?, g(&bv, 2)?);
                    Ok(Some(Value::Array(Rc::new(RefCell::new(vec![
                        Value::Float(a1 * b2 - a2 * b1),
                        Value::Float(a2 * b0 - a0 * b2),
                        Value::Float(a0 * b1 - a1 * b0),
                    ])))))
                }
                _ => Err(RuntimeError::new("외적: 1차원 배열 2개 필요", line)),
            }
        }
        "행렬합" => {
            if args.len() != 2 {
                return Err(RuntimeError::new("행렬합: 인자 2개 필요", line));
            }
            let a = eval_expr(&args[0], env, line)?;
            let b = eval_expr(&args[1], env, line)?;
            match (a, b) {
                (Value::Array(ar), Value::Array(br)) => {
                    let ar = ar.borrow();
                    let br = br.borrow();
                    if ar.len() != br.len() {
                        return Err(RuntimeError::new("행렬합: 행렬 크기가 같아야 합니다", line));
                    }
                    let mut result = Vec::with_capacity(ar.len());
                    for i in 0..ar.len() {
                        match (&ar[i], &br[i]) {
                            (Value::Array(a), Value::Array(b)) => {
                                let a = a.borrow();
                                let b = b.borrow();
                                let row: Vec<Value> = a
                                    .iter()
                                    .zip(b.iter())
                                    .map(|(x, y)| {
                                        let xv = match x {
                                            Value::Int(n) => *n as f64,
                                            Value::Float(f) => *f,
                                            _ => 0.0,
                                        };
                                        let yv = match y {
                                            Value::Int(n) => *n as f64,
                                            Value::Float(f) => *f,
                                            _ => 0.0,
                                        };
                                        Value::Float(xv + yv)
                                    })
                                    .collect();
                                result.push(Value::Array(Rc::new(RefCell::new(row))));
                            }
                            _ => return Err(RuntimeError::new("행렬합: 2차원 배열 필요", line)),
                        }
                    }
                    Ok(Some(Value::Array(Rc::new(RefCell::new(result)))))
                }
                _ => Err(RuntimeError::new("행렬합: 2차원 배열 필요", line)),
            }
        }
        "행렬차" => {
            if args.len() != 2 {
                return Err(RuntimeError::new("행렬차: 인자 2개 필요", line));
            }
            let a = eval_expr(&args[0], env, line)?;
            let b = eval_expr(&args[1], env, line)?;
            match (a, b) {
                (Value::Array(ar), Value::Array(br)) => {
                    let ar = ar.borrow();
                    let br = br.borrow();
                    if ar.len() != br.len() {
                        return Err(RuntimeError::new("행렬차: 행렬 크기가 같아야 합니다", line));
                    }
                    let mut result = Vec::with_capacity(ar.len());
                    for i in 0..ar.len() {
                        match (&ar[i], &br[i]) {
                            (Value::Array(a), Value::Array(b)) => {
                                let a = a.borrow();
                                let b = b.borrow();
                                let row: Vec<Value> = a
                                    .iter()
                                    .zip(b.iter())
                                    .map(|(x, y)| {
                                        let xv = match x {
                                            Value::Int(n) => *n as f64,
                                            Value::Float(f) => *f,
                                            _ => 0.0,
                                        };
                                        let yv = match y {
                                            Value::Int(n) => *n as f64,
                                            Value::Float(f) => *f,
                                            _ => 0.0,
                                        };
                                        Value::Float(xv - yv)
                                    })
                                    .collect();
                                result.push(Value::Array(Rc::new(RefCell::new(row))));
                            }
                            _ => return Err(RuntimeError::new("행렬차: 2차원 배열 필요", line)),
                        }
                    }
                    Ok(Some(Value::Array(Rc::new(RefCell::new(result)))))
                }
                _ => Err(RuntimeError::new("행렬차: 2차원 배열 필요", line)),
            }
        }
        "단위행렬" => {
            if args.len() != 1 {
                return Err(RuntimeError::new("단위행렬: 인자 1개 필요 (크기)", line));
            }
            let v = eval_expr(&args[0], env, line)?;
            let n = match v {
                Value::Int(n) => n as usize,
                _ => return Err(RuntimeError::new("단위행렬: 정수 필요", line)),
            };
            let mut result = Vec::with_capacity(n);
            for i in 0..n {
                let mut row = vec![Value::Float(0.0); n];
                row[i] = Value::Float(1.0);
                result.push(Value::Array(Rc::new(RefCell::new(row))));
            }
            Ok(Some(Value::Array(Rc::new(RefCell::new(result)))))
        }
        "텐서곱" => {
            if args.len() != 2 {
                return Err(RuntimeError::new("텐서곱: 인자 2개 필요", line));
            }
            let a = eval_expr(&args[0], env, line)?;
            let b = eval_expr(&args[1], env, line)?;
            match (a, b) {
                (Value::Array(a_rows), Value::Array(b_rows)) => {
                    let a_rows = a_rows.borrow();
                    let b_rows = b_rows.borrow();
                    let am = a_rows.len();
                    let bm = b_rows.len();
                    let an = match &a_rows[0] {
                        Value::Array(r) => r.borrow().len(),
                        _ => return Err(RuntimeError::new("텐서곱: 2차원 배열 필요", line)),
                    };
                    let bn = match &b_rows[0] {
                        Value::Array(r) => r.borrow().len(),
                        _ => return Err(RuntimeError::new("텐서곱: 2차원 배열 필요", line)),
                    };
                    let mut result = Vec::with_capacity(am * bm);
                    for i in 0..am {
                        let a_row = match &a_rows[i] {
                            Value::Array(r) => r.borrow().clone(),
                            _ => return Err(RuntimeError::new("텐서곱: 2차원 배열 필요", line)),
                        };
                        for k in 0..bm {
                            let b_row = match &b_rows[k] {
                                Value::Array(r) => r.borrow().clone(),
                                _ => {
                                    return Err(RuntimeError::new("텐서곱: 2차원 배열 필요", line));
                                }
                            };
                            let mut row = Vec::with_capacity(an * bn);
                            for a_val in a_row.iter().take(an) {
                                let av = match a_val {
                                    Value::Int(n) => *n as f64,
                                    Value::Float(f) => *f,
                                    _ => 0.0,
                                };
                                for b_val in b_row.iter().take(bn) {
                                    let bv = match b_val {
                                        Value::Int(n) => *n as f64,
                                        Value::Float(f) => *f,
                                        _ => 0.0,
                                    };
                                    row.push(Value::Float(av * bv));
                                }
                            }
                            result.push(Value::Array(Rc::new(RefCell::new(row))));
                        }
                    }
                    Ok(Some(Value::Array(Rc::new(RefCell::new(result)))))
                }
                _ => Err(RuntimeError::new("텐서곱: 2차원 배열 필요", line)),
            }
        }
        _ => Ok(None),
    }
}
