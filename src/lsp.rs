use std::collections::HashMap;
use std::io::{BufRead, Write};

const KEYWORDS: &[(&str, &str)] = &[
    (
        "함수",
        "function definition — `함수 이름(params) -> 반환타입 { ... }`",
    ),
    ("반환", "return value from function"),
    ("변수", "mutable variable declaration — `변수 이름 = 값`"),
    ("상수", "immutable constant — `상수 이름 = 값`"),
    (
        "만약",
        "if conditional — `만약 조건 { ... } 아니면 { ... }`",
    ),
    ("아니면", "else branch"),
    (
        "반복",
        "for loop — `반복 변수 i = 0; i < n; i += 1 { ... }`",
    ),
    ("동안", "while loop — `동안 조건 { ... }`"),
    ("멈춰", "break out of loop"),
    ("계속", "continue to next iteration"),
    ("참", "boolean true"),
    ("거짓", "boolean false"),
    ("없음", "void / null value"),
    ("출력", "print to stdout — `출력(값)`"),
    ("입력", "read line from stdin — `입력()`"),
    ("구조", "struct definition — `구조 이름 { 필드: 타입 }`"),
    ("시도", "try block — `시도 { ... } 실패(오류) { ... }`"),
    ("실패", "catch block"),
    ("가져오기", "import module — `가져오기 \"파일.hgl\"`"),
    ("맞춰", "pattern match — `맞춰 값 { 패턴 => 결과 }`"),
    ("구현", "impl block — `구현 구조체이름 { 함수들 }`"),
    ("정수", "64-bit integer type"),
    ("실수", "64-bit float type"),
    ("문자열", "UTF-8 string type"),
    ("불", "boolean type"),
];

const BUILTINS: &[(&str, &str)] = &[
    ("제곱근", "제곱근(x) — square root of x"),
    ("절댓값", "절댓값(x) — absolute value of x"),
    ("거듭제곱", "거듭제곱(밑, 지수) — base^exponent"),
    ("정수변환", "정수변환(x) — convert to integer"),
    ("실수변환", "실수변환(x) — convert to float"),
    ("길이", "길이(s) — length of string"),
    ("파일읽기", "파일읽기(경로) — read file to string"),
    ("파일쓰기", "파일쓰기(경로, 내용) — write string to file"),
    ("파일추가", "파일추가(경로, 내용) — append string to file"),
    ("파일존재", "파일존재(경로) — returns bool if file exists"),
    (
        "형식",
        "형식(\"template {0}\", 값) — format string with positional or named args",
    ),
    ("출력오류", "출력오류(값) — print to stderr"),
];

fn keyword_docs() -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (kw, doc) in KEYWORDS {
        map.insert(kw.to_string(), doc.to_string());
    }
    for (bi, doc) in BUILTINS {
        map.insert(bi.to_string(), doc.to_string());
    }
    map
}

fn read_message(reader: &mut impl BufRead) -> Option<String> {
    let mut content_length: usize = 0;
    let mut line = String::new();

    loop {
        line.clear();
        reader.read_line(&mut line).ok()?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some(rest) = trimmed.strip_prefix("Content-Length: ") {
            content_length = rest.parse().ok()?;
        }
    }

    if content_length == 0 {
        return None;
    }

    let mut buf = vec![0u8; content_length];
    let mut total = 0;
    while total < content_length {
        let n = std::io::Read::read(reader, &mut buf[total..]).ok()?;
        if n == 0 {
            break;
        }
        total += n;
    }
    String::from_utf8(buf).ok()
}

fn send_response(writer: &mut impl Write, id: &serde_like::Value, result: serde_like::Value) {
    let body = format!(
        "{{\"jsonrpc\":\"2.0\",\"id\":{},\"result\":{}}}",
        id, result
    );
    let _ = write!(writer, "Content-Length: {}\r\n\r\n{}", body.len(), body);
    let _ = writer.flush();
}

mod serde_like {
    use std::fmt;

    #[derive(Clone)]
    pub enum Value {
        Null,
        Bool(bool),
        Int(i64),
        Str(String),
        Array(Vec<Value>),
        Object(Vec<(String, Value)>),
    }

    impl fmt::Display for Value {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Value::Null => write!(f, "null"),
                Value::Bool(b) => write!(f, "{}", b),
                Value::Int(n) => write!(f, "{}", n),
                Value::Str(s) => {
                    let escaped = s
                        .replace('\\', "\\\\")
                        .replace('"', "\\\"")
                        .replace('\n', "\\n");
                    write!(f, "\"{}\"", escaped)
                }
                Value::Array(arr) => {
                    let items: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                    write!(f, "[{}]", items.join(","))
                }
                Value::Object(fields) => {
                    let pairs: Vec<String> = fields
                        .iter()
                        .map(|(k, v)| format!("\"{}\":{}", k, v))
                        .collect();
                    write!(f, "{{{}}}", pairs.join(","))
                }
            }
        }
    }

    pub fn parse_str(s: &str) -> Option<Value> {
        let s = s.trim();
        if s == "null" {
            return Some(Value::Null);
        }
        if s == "true" {
            return Some(Value::Bool(true));
        }
        if s == "false" {
            return Some(Value::Bool(false));
        }
        if let Ok(n) = s.parse::<i64>() {
            return Some(Value::Int(n));
        }
        if s.starts_with('"') && s.ends_with('"') {
            return Some(Value::Str(s[1..s.len() - 1].to_string()));
        }
        None
    }
}

fn parse_json_field<'a>(json: &'a str, field: &str) -> Option<&'a str> {
    let key = format!("\"{}\"", field);
    let pos = json.find(key.as_str())?;
    let after = json[pos + key.len()..].trim_start();
    let after = after.strip_prefix(':')?.trim_start();
    if after.starts_with('"') {
        let end = after[1..].find('"')? + 1;
        Some(&after[1..end])
    } else {
        let end = after
            .find(|c: char| ",}]".contains(c))
            .unwrap_or(after.len());
        Some(after[..end].trim())
    }
}

pub fn run_lsp() {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = std::io::BufReader::new(stdin.lock());
    let mut writer = std::io::BufWriter::new(stdout.lock());
    let docs = keyword_docs();

    loop {
        let msg = match read_message(&mut reader) {
            Some(m) => m,
            None => break,
        };

        let method = parse_json_field(&msg, "method").unwrap_or("").to_string();
        let id_str = parse_json_field(&msg, "id").unwrap_or("null").to_string();
        let id = serde_like::parse_str(&id_str).unwrap_or(serde_like::Value::Null);

        match method.as_str() {
            "initialize" => {
                let result = serde_like::Value::Object(vec![
                    (
                        "capabilities".to_string(),
                        serde_like::Value::Object(vec![
                            ("hoverProvider".to_string(), serde_like::Value::Bool(true)),
                            (
                                "completionProvider".to_string(),
                                serde_like::Value::Object(vec![(
                                    "triggerCharacters".to_string(),
                                    serde_like::Value::Array(vec![serde_like::Value::Str(
                                        ".".to_string(),
                                    )]),
                                )]),
                            ),
                            ("textDocumentSync".to_string(), serde_like::Value::Int(1)),
                        ]),
                    ),
                    (
                        "serverInfo".to_string(),
                        serde_like::Value::Object(vec![
                            (
                                "name".to_string(),
                                serde_like::Value::Str("han-lsp".to_string()),
                            ),
                            (
                                "version".to_string(),
                                serde_like::Value::Str("0.1.0".to_string()),
                            ),
                        ]),
                    ),
                ]);
                send_response(&mut writer, &id, result);
            }

            "textDocument/hover" => {
                let word = extract_word_at_cursor(&msg);
                let contents = if let Some(doc) = word.as_ref().and_then(|w| docs.get(w)) {
                    serde_like::Value::Object(vec![
                        (
                            "kind".to_string(),
                            serde_like::Value::Str("markdown".to_string()),
                        ),
                        (
                            "value".to_string(),
                            serde_like::Value::Str(format!(
                                "**{}** — {}",
                                word.as_deref().unwrap_or(""),
                                doc
                            )),
                        ),
                    ])
                } else {
                    serde_like::Value::Null
                };
                let result = if matches!(contents, serde_like::Value::Null) {
                    serde_like::Value::Null
                } else {
                    serde_like::Value::Object(vec![("contents".to_string(), contents)])
                };
                send_response(&mut writer, &id, result);
            }

            "textDocument/completion" => {
                let items: Vec<serde_like::Value> = KEYWORDS
                    .iter()
                    .chain(BUILTINS.iter())
                    .map(|(kw, doc)| {
                        serde_like::Value::Object(vec![
                            ("label".to_string(), serde_like::Value::Str(kw.to_string())),
                            (
                                "detail".to_string(),
                                serde_like::Value::Str(doc.to_string()),
                            ),
                            ("kind".to_string(), serde_like::Value::Int(14)),
                        ])
                    })
                    .collect();
                send_response(&mut writer, &id, serde_like::Value::Array(items));
            }

            "shutdown" => {
                send_response(&mut writer, &id, serde_like::Value::Null);
            }
            "exit" => break,
            _ => {
                if !method.is_empty()
                    && !method.starts_with("$/")
                    && !method.contains("notification")
                {
                    let result = serde_like::Value::Null;
                    send_response(&mut writer, &id, result);
                }
            }
        }
    }
}

fn extract_word_at_cursor(msg: &str) -> Option<String> {
    let text = parse_json_field(msg, "textDocument").or_else(|| parse_json_field(msg, "text"))?;
    let line_num: usize = parse_json_field(msg, "line")?.parse().ok()?;
    let char_num: usize = parse_json_field(msg, "character")?.parse().ok()?;

    let line = text.lines().nth(line_num)?;
    let chars: Vec<char> = line.chars().collect();
    if char_num >= chars.len() {
        return None;
    }
    let mut start = char_num;
    while start > 0 && (chars[start - 1].is_alphanumeric() || is_hangul(chars[start - 1])) {
        start -= 1;
    }
    let mut end = char_num;
    while end < chars.len() && (chars[end].is_alphanumeric() || is_hangul(chars[end])) {
        end += 1;
    }
    if start == end {
        return None;
    }
    Some(chars[start..end].iter().collect())
}

fn is_hangul(c: char) -> bool {
    ('\u{AC00}'..='\u{D7A3}').contains(&c)
        || ('\u{1100}'..='\u{11FF}').contains(&c)
        || ('\u{3130}'..='\u{318F}').contains(&c)
}
