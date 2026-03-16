use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // 키워드 (Keywords)
    함수,
    반환,
    변수,
    상수,
    만약,
    아니면,
    반복,
    동안,
    멈춰,
    계속,
    참,
    거짓,
    없음,
    출력,
    입력,
    구조,
    시도,
    처리,
    포함,
    맞춤,
    열거,
    안에서,
    DotDot,     // ..
    화살표이중, // =>
    구현,
    // 타입 키워드
    정수타입,
    실수타입,
    문자열타입,
    불타입,
    #[allow(dead_code)]
    없음타입,
    // 리터럴
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    // 식별자
    Identifier(String),
    // 산술 연산자
    Plus,    // +
    Minus,   // -
    Star,    // *
    Slash,   // /
    Percent, // %
    // 비교 연산자
    EqEq,   // ==
    BangEq, // !=
    Lt,     // <
    Gt,     // >
    LtEq,   // <=
    GtEq,   // >=
    // 논리 연산자
    AmpAmp,   // &&
    PipePipe, // ||
    Bang,     // !
    // 할당 연산자
    Eq,      // =
    PlusEq,  // +=
    MinusEq, // -=
    StarEq,  // *=
    SlashEq, // /=
    // 화살표
    Arrow, // ->
    // 구분자
    Colon,      // :
    ColonColon, // ::
    Comma,      // ,
    Semicolon,  // ;
    // 괄호
    LBrace,   // {
    RBrace,   // }
    LParen,   // (
    RParen,   // )
    LBracket, // [
    RBracket, // ]
    Dot,      // .
    // 특수
    Newline,
    Eof,
}

#[derive(Debug, Clone)]
pub struct TokenWithPos {
    pub token: Token,
    pub line: usize,
    pub col: usize,
}

impl TokenWithPos {
    pub fn new(token: Token, line: usize, col: usize) -> Self {
        Self { token, line, col }
    }
}

pub fn get_keyword_map() -> HashMap<String, Token> {
    let mut map = HashMap::new();
    // 키워드
    map.insert("함수".to_string(), Token::함수);
    map.insert("반환".to_string(), Token::반환);
    map.insert("변수".to_string(), Token::변수);
    map.insert("상수".to_string(), Token::상수);
    map.insert("만약".to_string(), Token::만약);
    map.insert("아니면".to_string(), Token::아니면);
    map.insert("반복".to_string(), Token::반복);
    map.insert("동안".to_string(), Token::동안);
    map.insert("멈춰".to_string(), Token::멈춰);
    map.insert("계속".to_string(), Token::계속);
    map.insert("참".to_string(), Token::참);
    map.insert("거짓".to_string(), Token::거짓);
    map.insert("없음".to_string(), Token::없음);
    map.insert("출력".to_string(), Token::출력);
    map.insert("입력".to_string(), Token::입력);
    map.insert("구조".to_string(), Token::구조);
    map.insert("시도".to_string(), Token::시도);
    map.insert("처리".to_string(), Token::처리);
    map.insert("포함".to_string(), Token::포함);
    map.insert("맞춤".to_string(), Token::맞춤);
    map.insert("구현".to_string(), Token::구현);
    map.insert("열거".to_string(), Token::열거);
    map.insert("안에서".to_string(), Token::안에서);
    // 타입 키워드
    map.insert("정수".to_string(), Token::정수타입);
    map.insert("실수".to_string(), Token::실수타입);
    map.insert("문자열".to_string(), Token::문자열타입);
    map.insert("불".to_string(), Token::불타입);
    map
}

fn is_korean(c: char) -> bool {
    ('\u{AC00}'..='\u{D7A3}').contains(&c)   // 완성형 한글
        || ('\u{1100}'..='\u{11FF}').contains(&c) // 자모
        || ('\u{3130}'..='\u{318F}').contains(&c) // 호환자모
}

fn is_identifier_start(c: char) -> bool {
    c.is_alphabetic() || c == '_' || is_korean(c)
}

fn is_identifier_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || is_korean(c)
}

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    keywords: HashMap<String, Token>,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 0,
            keywords: get_keyword_map(),
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied()?;
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 0;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        // consume until newline (don't consume the newline itself)
        while let Some(c) = self.peek() {
            if c == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn read_string(&mut self) -> Result<String, String> {
        // opening `"` already consumed
        let mut s = String::new();
        loop {
            match self.advance() {
                None => return Err("Unterminated string literal".to_string()),
                Some('"') => break,
                Some('\\') => match self.advance() {
                    Some('n') => s.push('\n'),
                    Some('t') => s.push('\t'),
                    Some('\\') => s.push('\\'),
                    Some('"') => s.push('"'),
                    Some(c) => {
                        s.push('\\');
                        s.push(c);
                    }
                    None => return Err("Unterminated escape".to_string()),
                },
                Some(c) => s.push(c),
            }
        }
        Ok(s)
    }

    fn read_number(&mut self, first: char) -> Token {
        let mut num = String::new();
        num.push(first);
        let mut is_float = false;

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                num.push(c);
                self.advance();
            } else if c == '.' && !is_float && self.peek_next().is_some_and(|n| n.is_ascii_digit())
            {
                is_float = true;
                num.push(c);
                self.advance();
            } else {
                break;
            }
        }

        if is_float {
            Token::FloatLiteral(num.parse().unwrap_or(0.0))
        } else {
            Token::IntLiteral(num.parse().unwrap_or(0))
        }
    }

    fn read_identifier(&mut self, first: char) -> Token {
        let mut ident = String::new();
        ident.push(first);
        while let Some(c) = self.peek() {
            if is_identifier_continue(c) {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }
        self.keywords
            .get(&ident)
            .cloned()
            .unwrap_or(Token::Identifier(ident))
    }

    fn next_token(&mut self) -> Option<TokenWithPos> {
        loop {
            self.skip_whitespace();

            if self.peek() == Some('/') && self.peek_next() == Some('/') {
                self.advance();
                self.advance();
                self.skip_line_comment();
                continue;
            }

            break;
        }

        let line = self.line;
        let col = self.col;

        let c = self.advance()?;

        let token = match c {
            '\n' => Token::Newline,

            '"' => match self.read_string() {
                Ok(s) => Token::StringLiteral(s),
                Err(_) => return None,
            },

            '0'..='9' => self.read_number(c),

            '+' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::PlusEq
                } else {
                    Token::Plus
                }
            }
            '-' => {
                if self.peek() == Some('>') {
                    self.advance();
                    Token::Arrow
                } else if self.peek() == Some('=') {
                    self.advance();
                    Token::MinusEq
                } else {
                    Token::Minus
                }
            }
            '*' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::StarEq
                } else {
                    Token::Star
                }
            }
            '/' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::SlashEq
                } else {
                    Token::Slash
                }
            }
            '%' => Token::Percent,
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::EqEq
                } else if self.peek() == Some('>') {
                    self.advance();
                    Token::화살표이중
                } else {
                    Token::Eq
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::BangEq
                } else {
                    Token::Bang
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::LtEq
                } else {
                    Token::Lt
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::GtEq
                } else {
                    Token::Gt
                }
            }
            '&' => {
                if self.peek() == Some('&') {
                    self.advance();
                    Token::AmpAmp
                } else {
                    return None;
                }
            }
            '|' => {
                if self.peek() == Some('|') {
                    self.advance();
                    Token::PipePipe
                } else {
                    return None;
                }
            }
            ':' => {
                if self.peek() == Some(':') {
                    self.advance();
                    Token::ColonColon
                } else {
                    Token::Colon
                }
            }
            ',' => Token::Comma,
            ';' => Token::Semicolon,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '(' => Token::LParen,
            ')' => Token::RParen,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            '.' => {
                if self.peek() == Some('.') {
                    self.advance();
                    Token::DotDot
                } else {
                    Token::Dot
                }
            }

            c if is_identifier_start(c) => self.read_identifier(c),

            _ => return None,
        };

        Some(TokenWithPos::new(token, line, col))
    }
}

pub fn tokenize(source: &str) -> Vec<TokenWithPos> {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();
    while let Some(tok) = lexer.next_token() {
        tokens.push(tok);
    }
    tokens.push(TokenWithPos::new(Token::Eof, lexer.line, lexer.col));
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(src: &str) -> Vec<Token> {
        tokenize(src)
            .into_iter()
            .filter(|t| !matches!(t.token, Token::Newline | Token::Eof))
            .map(|t| t.token)
            .collect()
    }

    #[test]
    fn test_keyword_map() {
        let map = get_keyword_map();
        assert_eq!(map.get("함수"), Some(&Token::함수));
        assert_eq!(map.get("반환"), Some(&Token::반환));
        assert_eq!(map.get("만약"), Some(&Token::만약));
        assert_eq!(map.get("정수"), Some(&Token::정수타입));
    }

    #[test]
    fn test_token_with_pos() {
        let t = TokenWithPos::new(Token::함수, 1, 0);
        assert_eq!(t.line, 1);
        assert_eq!(t.col, 0);
        assert!(matches!(t.token, Token::함수));
    }

    #[test]
    fn test_korean_keyword() {
        let toks = tokens("함수");
        assert_eq!(toks, vec![Token::함수]);
    }

    #[test]
    fn test_korean_identifier() {
        let toks = tokens("나이");
        assert_eq!(toks, vec![Token::Identifier("나이".to_string())]);
    }

    #[test]
    fn test_integer_literal() {
        let toks = tokens("42");
        assert_eq!(toks, vec![Token::IntLiteral(42)]);
    }

    #[test]
    fn test_float_literal() {
        let toks = tokens("3.14");
        assert_eq!(toks, vec![Token::FloatLiteral(3.14)]);
    }

    #[test]
    fn test_string_literal() {
        let toks = tokens("\"안녕\"");
        assert_eq!(toks, vec![Token::StringLiteral("안녕".to_string())]);
    }

    #[test]
    fn test_operators() {
        let toks = tokens("+ == -> += !=");
        assert_eq!(
            toks,
            vec![
                Token::Plus,
                Token::EqEq,
                Token::Arrow,
                Token::PlusEq,
                Token::BangEq,
            ]
        );
    }

    #[test]
    fn test_full_function() {
        let src = "함수 더하기(가: 정수, 나: 정수) -> 정수 { 반환 가 + 나 }";
        let toks = tokens(src);
        assert_eq!(
            toks,
            vec![
                Token::함수,
                Token::Identifier("더하기".to_string()),
                Token::LParen,
                Token::Identifier("가".to_string()),
                Token::Colon,
                Token::정수타입,
                Token::Comma,
                Token::Identifier("나".to_string()),
                Token::Colon,
                Token::정수타입,
                Token::RParen,
                Token::Arrow,
                Token::정수타입,
                Token::LBrace,
                Token::반환,
                Token::Identifier("가".to_string()),
                Token::Plus,
                Token::Identifier("나".to_string()),
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn test_line_comment_skipped() {
        let toks = tokens("42 // 이건 주석\n99");
        assert_eq!(toks, vec![Token::IntLiteral(42), Token::IntLiteral(99)]);
    }

    #[test]
    fn test_position_tracking() {
        let result = tokenize("함수");
        assert_eq!(result[0].line, 1);
        assert_eq!(result[0].col, 0);
    }
}
