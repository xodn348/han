use crate::ast::{Program, StmtKind};
use crate::lexer;
use crate::parser;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
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
    ("시도", "try block — `시도 { ... } 처리(오류) { ... }`"),
    ("실패", "catch block"),
    ("포함", "import module — `포함 \"파일.hgl\"`"),
    ("맞춤", "pattern match — `맞춤 값 { 패턴 => 결과 }`"),
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

#[derive(Deserialize)]
struct IncomingMessage {
    #[serde(default)]
    id: Option<Value>,
    #[serde(default)]
    method: Option<String>,
    #[serde(default)]
    params: Value,
}

#[derive(Serialize)]
struct ResponseMessage<'a> {
    jsonrpc: &'a str,
    id: &'a Value,
    result: Value,
}

fn send_response(writer: &mut impl Write, id: &Value, result: Value) {
    let response = ResponseMessage {
        jsonrpc: "2.0",
        id,
        result,
    };
    let body = match serde_json::to_string(&response) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("[han-lsp] failed to serialize response: {}", e);
            return;
        }
    };
    let _ = write!(writer, "Content-Length: {}\r\n\r\n{}", body.len(), body);
    let _ = writer.flush();
}

struct LspServer {
    docs: HashMap<String, String>,
    current_source: String,
    cached: Option<CachedDoc>,
}

struct CachedDoc {
    source_hash: u64,
    program: Option<Program>,
    symbols: Vec<SourceSymbol>,
}

impl LspServer {
    fn new() -> Self {
        Self {
            docs: keyword_docs(),
            current_source: String::new(),
            cached: None,
        }
    }

    fn doc_symbols(&self) -> &[SourceSymbol] {
        self.cached
            .as_ref()
            .map(|c| c.symbols.as_slice())
            .unwrap_or(&[])
    }

    /// Update the active source. Reuses the cached AST/symbols when the
    /// source bytes are unchanged so per-keystroke notifications don't
    /// trigger redundant lex+parse work.
    fn update_source(&mut self, text: String) {
        let hash = hash_str(&text);
        if let Some(cached) = &self.cached
            && cached.source_hash == hash
        {
            self.current_source = text;
            return;
        }

        let tokens = lexer::tokenize(&text);
        let (program, symbols) = match parser::parse(tokens) {
            Ok(program) => {
                let symbols = extract_symbols_from_program(&program);
                (Some(program), symbols)
            }
            Err(_) => (None, Vec::new()),
        };

        self.cached = Some(CachedDoc {
            source_hash: hash,
            program,
            symbols,
        });
        self.current_source = text;
    }

    fn handle_did_open(&mut self, params: &Value) {
        if let Some(text) = params.pointer("/textDocument/text").and_then(Value::as_str) {
            self.update_source(text.to_string());
        }
    }

    fn handle_did_change(&mut self, params: &Value) {
        // textDocumentSync = 1 (Full): each change carries the entire document.
        if let Some(text) = params
            .pointer("/contentChanges/0/text")
            .and_then(Value::as_str)
        {
            self.update_source(text.to_string());
        }
    }

    fn extract_word_at_cursor(&self, params: &Value) -> Option<String> {
        let line_num = params.pointer("/position/line")?.as_u64()? as usize;
        let char_num = params.pointer("/position/character")?.as_u64()? as usize;
        let line = self.current_source.lines().nth(line_num)?;
        let chars: Vec<char> = line.chars().collect();
        if char_num > chars.len() {
            return None;
        }
        let mut start = char_num.min(chars.len());
        while start > 0 && is_word_char(chars[start - 1]) {
            start -= 1;
        }
        let mut end = char_num.min(chars.len());
        while end < chars.len() && is_word_char(chars[end]) {
            end += 1;
        }
        if start == end {
            return None;
        }
        Some(chars[start..end].iter().collect())
    }

    fn hover_result(&self, params: &Value) -> Value {
        let word = match self.extract_word_at_cursor(params) {
            Some(w) => w,
            None => return Value::Null,
        };

        if let Some(doc) = self.docs.get(&word) {
            return json!({
                "contents": {
                    "kind": "markdown",
                    "value": format!("**{}** — {}", word, doc),
                }
            });
        }
        if let Some(sym) = self.doc_symbols().iter().find(|s| s.name == word) {
            return json!({
                "contents": {
                    "kind": "markdown",
                    "value": format!("**{}** ({})\n\n`{}`", sym.name, sym.kind, sym.detail),
                }
            });
        }
        Value::Null
    }

    fn completion_result(&self) -> Value {
        let mut items: Vec<Value> = KEYWORDS
            .iter()
            .chain(BUILTINS.iter())
            .map(|(kw, doc)| {
                json!({
                    "label": kw,
                    "detail": doc,
                    "kind": 14,
                })
            })
            .collect();

        for sym in self.doc_symbols() {
            let kind = match sym.kind {
                "function" => 3,
                "variable" => 6,
                "struct" => 22,
                "enum" => 13,
                "enum_variant" => 20,
                "field" => 5,
                _ => 1,
            };
            items.push(json!({
                "label": sym.name,
                "detail": sym.detail,
                "kind": kind,
            }));
        }

        Value::Array(items)
    }

    fn diagnostic_result(&self) -> Value {
        let mut diags: Vec<Value> = Vec::new();

        // Fast path: the cached parse succeeded — no diagnostics, no re-parse.
        let needs_reparse = match &self.cached {
            Some(c) => c.program.is_none(),
            None => !self.current_source.is_empty(),
        };
        if needs_reparse {
            let tokens = lexer::tokenize(&self.current_source);
            if let Err(e) = parser::parse(tokens) {
                let line = e.line.saturating_sub(1) as i64;
                diags.push(json!({
                    "range": {
                        "start": { "line": line, "character": 0 },
                        "end":   { "line": line, "character": 100 },
                    },
                    "severity": 1,
                    "message": e.message,
                }));
            }
        }
        json!({
            "kind": "full",
            "items": diags,
        })
    }
}

fn hash_str(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || is_hangul(c)
}

fn initialize_result() -> Value {
    json!({
        "capabilities": {
            "textDocumentSync": 1,
            "hoverProvider": true,
            "completionProvider": {
                "triggerCharacters": ["."],
            },
            "diagnosticProvider": {
                "interFileDependencies": false,
                "workspaceDiagnostics": false,
            },
            "definitionProvider": false,
            "referencesProvider": false,
            "renameProvider": false,
            "workspaceSymbolProvider": false,
            "codeActionProvider": false,
        },
        "serverInfo": {
            "name": "han-lsp",
            "version": "0.1.0",
        },
    })
}

pub fn run_lsp() {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = std::io::BufReader::new(stdin.lock());
    let mut writer = std::io::BufWriter::new(stdout.lock());
    let mut server = LspServer::new();

    while let Some(raw) = read_message(&mut reader) {
        let msg: IncomingMessage = match serde_json::from_str(&raw) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("[han-lsp] failed to parse incoming message: {}", e);
                continue;
            }
        };

        let id = msg.id.unwrap_or(Value::Null);
        let method = msg.method.as_deref().unwrap_or("");
        let params = &msg.params;

        match method {
            "initialize" => {
                send_response(&mut writer, &id, initialize_result());
            }
            "initialized" | "$/cancelRequest" => {
                // notification; no response.
            }
            "textDocument/didOpen" => {
                server.handle_did_open(params);
            }
            "textDocument/didChange" => {
                server.handle_did_change(params);
            }
            "textDocument/didClose" | "textDocument/didSave" => {
                // notifications; no response.
            }
            "textDocument/hover" => {
                let result = server.hover_result(params);
                send_response(&mut writer, &id, result);
            }
            "textDocument/completion" => {
                send_response(&mut writer, &id, server.completion_result());
            }
            "textDocument/diagnostic" => {
                send_response(&mut writer, &id, server.diagnostic_result());
            }
            // Declared as unsupported in `initialize`, but some clients still
            // send them. Respond with explicit null + a log line so the
            // mismatch is visible.
            "textDocument/definition"
            | "textDocument/references"
            | "textDocument/rename"
            | "textDocument/codeAction"
            | "workspace/symbol" => {
                eprintln!("[han-lsp] unimplemented method: {}", method);
                send_response(&mut writer, &id, Value::Null);
            }
            "shutdown" => {
                send_response(&mut writer, &id, Value::Null);
            }
            "exit" => break,
            "" => {
                // response or malformed — ignore.
            }
            _ => {
                if !method.starts_with("$/") && !id.is_null() {
                    send_response(&mut writer, &id, Value::Null);
                }
            }
        }
    }
}

struct SourceSymbol {
    name: String,
    kind: &'static str,
    detail: String,
}

fn extract_symbols_from_program(program: &Program) -> Vec<SourceSymbol> {
    let mut symbols = Vec::new();
    for stmt in &program.stmts {
        match &stmt.kind {
            StmtKind::FuncDef {
                name,
                params,
                return_type,
                ..
            } => {
                let params_str: Vec<String> = params
                    .iter()
                    .map(|(n, t)| format!("{}: {:?}", n, t))
                    .collect();
                let ret = return_type
                    .as_ref()
                    .map(|t| format!(" -> {:?}", t))
                    .unwrap_or_default();
                symbols.push(SourceSymbol {
                    name: name.clone(),
                    kind: "function",
                    detail: format!("함수 {}({}){}", name, params_str.join(", "), ret),
                });
            }
            StmtKind::VarDecl {
                name, ty, mutable, ..
            } => {
                let prefix = if *mutable { "변수" } else { "상수" };
                let ty_str = ty
                    .as_ref()
                    .map(|t| format!(": {:?}", t))
                    .unwrap_or_default();
                symbols.push(SourceSymbol {
                    name: name.clone(),
                    kind: "variable",
                    detail: format!("{} {}{}", prefix, name, ty_str),
                });
            }
            StmtKind::StructDef { name, fields } => {
                let fields_str: Vec<String> = fields
                    .iter()
                    .map(|(n, t)| format!("{}: {:?}", n, t))
                    .collect();
                symbols.push(SourceSymbol {
                    name: name.clone(),
                    kind: "struct",
                    detail: format!("구조 {} {{ {} }}", name, fields_str.join(", ")),
                });
                for (fname, ftype) in fields {
                    symbols.push(SourceSymbol {
                        name: fname.clone(),
                        kind: "field",
                        detail: format!("{}.{}: {:?}", name, fname, ftype),
                    });
                }
            }
            StmtKind::EnumDef { name, variants } => {
                symbols.push(SourceSymbol {
                    name: name.clone(),
                    kind: "enum",
                    detail: format!("열거 {} {{ {} }}", name, variants.join(", ")),
                });
                for v in variants {
                    symbols.push(SourceSymbol {
                        name: format!("{}::{}", name, v),
                        kind: "enum_variant",
                        detail: format!("{}::{}", name, v),
                    });
                }
            }
            _ => {}
        }
    }
    symbols
}

fn is_hangul(c: char) -> bool {
    ('\u{AC00}'..='\u{D7A3}').contains(&c)
        || ('\u{1100}'..='\u{11FF}').contains(&c)
        || ('\u{3130}'..='\u{318F}').contains(&c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caches_ast_when_source_unchanged() {
        let mut server = LspServer::new();
        server.update_source("함수 더하기(a: 정수, b: 정수) -> 정수 { 반환 a + b }".to_string());
        let first_hash = server.cached.as_ref().unwrap().source_hash;
        let first_symbols_ptr = server.doc_symbols().as_ptr();

        // Identical source → cache should be reused, not rebuilt.
        server.update_source("함수 더하기(a: 정수, b: 정수) -> 정수 { 반환 a + b }".to_string());
        let second_hash = server.cached.as_ref().unwrap().source_hash;
        let second_symbols_ptr = server.doc_symbols().as_ptr();

        assert_eq!(first_hash, second_hash);
        assert_eq!(first_symbols_ptr, second_symbols_ptr);
    }

    #[test]
    fn rebuilds_symbols_on_source_change() {
        let mut server = LspServer::new();
        server.update_source("함수 첫번째() -> 정수 { 반환 1 }".to_string());
        assert!(server.doc_symbols().iter().any(|s| s.name == "첫번째"));

        server.update_source("함수 두번째() -> 정수 { 반환 2 }".to_string());
        assert!(server.doc_symbols().iter().any(|s| s.name == "두번째"));
        assert!(!server.doc_symbols().iter().any(|s| s.name == "첫번째"));
    }

    #[test]
    fn hover_returns_keyword_doc_via_cached_source() {
        let mut server = LspServer::new();
        server.update_source("함수 보기() -> 정수 { 반환 1 }".to_string());
        let params = json!({
            "textDocument": { "uri": "file:///x.hgl" },
            "position": { "line": 0, "character": 1 },
        });
        let result = server.hover_result(&params);
        let value = result
            .pointer("/contents/value")
            .and_then(Value::as_str)
            .unwrap_or("");
        assert!(value.contains("함수"), "hover value was: {}", value);
    }

    #[test]
    fn handles_strings_with_embedded_quotes_and_newlines() {
        // The previous hand-rolled parser failed on `\"` and `\n` escapes.
        // serde_json must round-trip them cleanly.
        let raw = r#"{"jsonrpc":"2.0","id":1,"method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///x.hgl","text":"변수 a = \"hi\\n\"\n출력(a)"}}}"#;
        let msg: IncomingMessage = serde_json::from_str(raw).expect("valid JSON");
        let mut server = LspServer::new();
        server.handle_did_open(&msg.params);
        // \" → "  ;  \\n → \n (literal backslash-n) ; \n → real newline
        assert!(server.current_source.contains("\"hi\\n\""));
        assert!(server.current_source.contains("\n출력(a)"));
    }
}
