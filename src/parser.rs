
use crate::ast::*;
use crate::lexer::{Token, TokenWithPos};

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
}

impl ParseError {
    fn new(message: impl Into<String>, line: usize) -> Self {
        Self {
            message: message.into(),
            line,
        }
    }
}

// ── Parser struct ─────────────────────────────────────────────────────────────

pub struct Parser {
    tokens: Vec<TokenWithPos>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<TokenWithPos>) -> Self {
        Self { tokens, pos: 0 }
    }

    // ── Cursor helpers ────────────────────────────────────────────────────────

    fn peek(&self) -> &Token {
        let mut i = self.pos;
        while i < self.tokens.len() {
            if !matches!(self.tokens[i].token, Token::Newline) {
                return &self.tokens[i].token;
            }
            i += 1;
        }
        &Token::Eof
    }

    fn peek_pos(&self) -> (usize, usize) {
        let mut i = self.pos;
        while i < self.tokens.len() {
            if !matches!(self.tokens[i].token, Token::Newline) {
                return (self.tokens[i].line, self.tokens[i].col);
            }
            i += 1;
        }
        (0, 0)
    }

    fn span_here(&self) -> Span {
        let (line, col) = self.peek_pos();
        Span::new(line, col)
    }

    fn advance(&mut self) -> &Token {
        while self.pos < self.tokens.len() && matches!(self.tokens[self.pos].token, Token::Newline)
        {
            self.pos += 1;
        }
        if self.pos < self.tokens.len() {
            let tok = &self.tokens[self.pos].token;
            self.pos += 1;
            tok
        } else {
            &Token::Eof
        }
    }

    fn expect(&mut self, expected: &Token) -> Result<(), ParseError> {
        let (line, _) = self.peek_pos();
        let tok = self.peek().clone();
        if std::mem::discriminant(&tok) == std::mem::discriminant(expected) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::new(
                format!("'{:?}' 예상, '{:?}' 발견", expected, tok),
                line,
            ))
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    // ── Top-level parse ───────────────────────────────────────────────────────

    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut stmts = Vec::new();
        while !self.is_at_end() {
            stmts.push(self.parse_stmt()?);
        }
        Ok(Program::new(stmts))
    }

    // ── Statement dispatch ────────────────────────────────────────────────────

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        let span = self.span_here();
        match self.peek().clone() {
            Token::함수 => self.parse_func_def(),
            Token::변수 => {
                self.advance();
                self.parse_var_decl(true, span)
            }
            Token::상수 => {
                self.advance();
                self.parse_var_decl(false, span)
            }
            Token::만약 => self.parse_if_stmt(),
            Token::반복 => self.parse_for_loop(),
            Token::동안 => self.parse_while_loop(),
            Token::반환 => {
                self.advance();
                if self.is_at_end()
                    || matches!(self.peek(), Token::RBrace | Token::Newline | Token::Eof)
                {
                    Ok(Stmt::new(StmtKind::Return(None), span))
                } else {
                    let expr = self.parse_expr()?;
                    Ok(Stmt::new(StmtKind::Return(Some(expr)), span))
                }
            }
            Token::멈춰 => {
                self.advance();
                Ok(Stmt::new(StmtKind::Break, span))
            }
            Token::계속 => {
                self.advance();
                Ok(Stmt::new(StmtKind::Continue, span))
            }
            _ => {
                let expr = self.parse_expr()?;
                Ok(Stmt::new(StmtKind::ExprStmt(expr), span))
            }
        }
    }

    // ── Function definition ───────────────────────────────────────────────────

    fn parse_func_def(&mut self) -> Result<Stmt, ParseError> {
        let span = self.span_here();
        self.advance();

        let name = match self.advance().clone() {
            Token::Identifier(n) => n,
            tok => {
                return Err(ParseError::new(
                    format!("함수 이름 예상, '{:?}' 발견", tok),
                    span.line,
                ))
            }
        };

        self.expect(&Token::LParen)?;

        let mut params: Vec<(String, Type)> = Vec::new();
        while !matches!(self.peek(), Token::RParen | Token::Eof) {
            let param_name = match self.advance().clone() {
                Token::Identifier(n) => n,
                tok => {
                    return Err(ParseError::new(
                        format!("매개변수 이름 예상, '{:?}' 발견", tok),
                        span.line,
                    ))
                }
            };
            self.expect(&Token::Colon)?;
            let ty = self.parse_type()?;
            params.push((param_name, ty));
            if matches!(self.peek(), Token::Comma) {
                self.advance();
            }
        }

        self.expect(&Token::RParen)?;

        let return_type = if matches!(self.peek(), Token::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = self.parse_block()?;

        Ok(Stmt::new(
            StmtKind::FuncDef {
                name,
                params,
                return_type,
                body,
            },
            span,
        ))
    }

    // ── Variable declaration ──────────────────────────────────────────────────

    fn parse_var_decl(&mut self, mutable: bool, span: Span) -> Result<Stmt, ParseError> {
        let name = match self.advance().clone() {
            Token::Identifier(n) => n,
            tok => {
                return Err(ParseError::new(
                    format!("변수 이름 예상, '{:?}' 발견", tok),
                    span.line,
                ))
            }
        };

        let ty = if matches!(self.peek(), Token::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(&Token::Eq)?;
        let value = self.parse_expr()?;

        Ok(Stmt::new(
            StmtKind::VarDecl {
                name,
                ty,
                value,
                mutable,
            },
            span,
        ))
    }

    // ── If statement ──────────────────────────────────────────────────────────

    fn parse_if_stmt(&mut self) -> Result<Stmt, ParseError> {
        let span = self.span_here();
        self.advance();
        let cond = self.parse_expr()?;
        let then_block = self.parse_block()?;

        let else_block = if matches!(self.peek(), Token::아니면) {
            self.advance();
            // 아니면 만약 → else-if chaining
            if matches!(self.peek(), Token::만약) {
                let nested_if = self.parse_if_stmt()?;
                Some(vec![nested_if])
            } else {
                Some(self.parse_block()?)
            }
        } else {
            None
        };

        Ok(Stmt::new(
            StmtKind::If {
                cond,
                then_block,
                else_block,
            },
            span,
        ))
    }

    // ── For loop ──────────────────────────────────────────────────────────────

    fn parse_for_loop(&mut self) -> Result<Stmt, ParseError> {
        let span = self.span_here();
        self.advance();

        let init_span = self.span_here();
        let init = if matches!(self.peek(), Token::변수) {
            self.advance();
            self.parse_var_decl(true, init_span)?
        } else if matches!(self.peek(), Token::상수) {
            self.advance();
            self.parse_var_decl(false, init_span)?
        } else {
            let expr = self.parse_expr()?;
            Stmt::new(StmtKind::ExprStmt(expr), init_span)
        };

        self.expect(&Token::Semicolon)?;
        let cond = self.parse_expr()?;
        self.expect(&Token::Semicolon)?;

        let step_span = self.span_here();
        let step_expr = self.parse_expr()?;
        let step = Stmt::new(StmtKind::ExprStmt(step_expr), step_span);

        let body = self.parse_block()?;

        Ok(Stmt::new(
            StmtKind::ForLoop {
                init: Box::new(init),
                cond,
                step: Box::new(step),
                body,
            },
            span,
        ))
    }

    // ── While loop ────────────────────────────────────────────────────────────

    fn parse_while_loop(&mut self) -> Result<Stmt, ParseError> {
        let span = self.span_here();
        self.advance();
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::new(StmtKind::WhileLoop { cond, body }, span))
    }

    // ── Block ─────────────────────────────────────────────────────────────────

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        self.expect(&Token::LBrace)?;
        let mut stmts = Vec::new();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(&Token::RBrace)?;
        Ok(stmts)
    }

    // ── Expression parsing (precedence climbing) ──────────────────────────────

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_or()?;

        let (line, _) = self.peek_pos();
        match self.peek().clone() {
            Token::Eq => {
                self.advance();
                let value = self.parse_assignment()?;
                if let Expr::Identifier(name) = left {
                    return Ok(Expr::Assign {
                        name,
                        value: Box::new(value),
                    });
                }
                Err(ParseError::new("할당 대상이 올바르지 않습니다", line))
            }
            Token::PlusEq | Token::MinusEq | Token::StarEq | Token::SlashEq => {
                let op_tok = self.advance().clone();
                let rhs = self.parse_assignment()?;
                if let Expr::Identifier(ref name) = left {
                    let op = match op_tok {
                        Token::PlusEq => BinaryOpKind::Add,
                        Token::MinusEq => BinaryOpKind::Sub,
                        Token::StarEq => BinaryOpKind::Mul,
                        Token::SlashEq => BinaryOpKind::Div,
                        _ => unreachable!(),
                    };
                    let compound = Expr::BinaryOp {
                        op,
                        left: Box::new(left.clone()),
                        right: Box::new(rhs),
                    };
                    return Ok(Expr::Assign {
                        name: name.clone(),
                        value: Box::new(compound),
                    });
                }
                Err(ParseError::new("복합 할당 대상이 올바르지 않습니다", line))
            }
            _ => Ok(left),
        }
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and()?;
        while matches!(self.peek(), Token::PipePipe) {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::BinaryOp {
                op: BinaryOpKind::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_comparison()?;
        while matches!(self.peek(), Token::AmpAmp) {
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::BinaryOp {
                op: BinaryOpKind::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_addition()?;
        loop {
            let op = match self.peek() {
                Token::EqEq => BinaryOpKind::Eq,
                Token::BangEq => BinaryOpKind::NotEq,
                Token::Lt => BinaryOpKind::Lt,
                Token::Gt => BinaryOpKind::Gt,
                Token::LtEq => BinaryOpKind::LtEq,
                Token::GtEq => BinaryOpKind::GtEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_addition()?;
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplication()?;
        loop {
            let op = match self.peek() {
                Token::Plus => BinaryOpKind::Add,
                Token::Minus => BinaryOpKind::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplication()?;
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Token::Star => BinaryOpKind::Mul,
                Token::Slash => BinaryOpKind::Div,
                Token::Percent => BinaryOpKind::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        match self.peek().clone() {
            Token::Minus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp {
                    op: UnaryOpKind::Neg,
                    expr: Box::new(expr),
                })
            }
            Token::Bang => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp {
                    op: UnaryOpKind::Not,
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let (line, _) = self.peek_pos();
        match self.peek().clone() {
            Token::IntLiteral(n) => {
                self.advance();
                Ok(Expr::IntLiteral(n))
            }
            Token::FloatLiteral(f) => {
                self.advance();
                Ok(Expr::FloatLiteral(f))
            }
            Token::StringLiteral(s) => {
                self.advance();
                Ok(Expr::StringLiteral(s))
            }
            Token::참 => {
                self.advance();
                Ok(Expr::BoolLiteral(true))
            }
            Token::거짓 => {
                self.advance();
                Ok(Expr::BoolLiteral(false))
            }
            Token::Identifier(name) => {
                self.advance();
                if matches!(self.peek(), Token::LParen) {
                    self.advance();
                    let mut args = Vec::new();
                    while !matches!(self.peek(), Token::RParen | Token::Eof) {
                        args.push(self.parse_expr()?);
                        if matches!(self.peek(), Token::Comma) {
                            self.advance();
                        }
                    }
                    self.expect(&Token::RParen)?;
                    Ok(Expr::Call { name, args })
                } else {
                    Ok(Expr::Identifier(name))
                }
            }
            Token::출력 => {
                self.advance();
                self.expect(&Token::LParen)?;
                let mut args = Vec::new();
                while !matches!(self.peek(), Token::RParen | Token::Eof) {
                    args.push(self.parse_expr()?);
                    if matches!(self.peek(), Token::Comma) {
                        self.advance();
                    }
                }
                self.expect(&Token::RParen)?;
                Ok(Expr::Call {
                    name: "출력".to_string(),
                    args,
                })
            }
            Token::입력 => {
                self.advance();
                self.expect(&Token::LParen)?;
                let mut args = Vec::new();
                while !matches!(self.peek(), Token::RParen | Token::Eof) {
                    args.push(self.parse_expr()?);
                    if matches!(self.peek(), Token::Comma) {
                        self.advance();
                    }
                }
                self.expect(&Token::RParen)?;
                Ok(Expr::Call {
                    name: "입력".to_string(),
                    args,
                })
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            tok => Err(ParseError::new(
                format!("표현식에서 예상치 못한 토큰: {:?}", tok),
                line,
            )),
        }
    }

    // ── Type parsing ──────────────────────────────────────────────────────────

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        let (line, _) = self.peek_pos();
        match self.advance().clone() {
            Token::정수타입 => Ok(Type::정수),
            Token::실수타입 => Ok(Type::실수),
            Token::문자열타입 => Ok(Type::문자열),
            Token::불타입 => Ok(Type::불),
            Token::없음타입 => Ok(Type::없음),
            tok => Err(ParseError::new(
                format!("타입 예상 (정수/실수/문자열/불/없음), '{:?}' 발견", tok),
                line,
            )),
        }
    }
}

// ── Public entry point ────────────────────────────────────────────────────────

pub fn parse(tokens: Vec<TokenWithPos>) -> Result<Program, ParseError> {
    Parser::new(tokens).parse()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    fn parse_src(src: &str) -> Program {
        let tokens = tokenize(src);
        parse(tokens).expect("parse failed")
    }

    #[test]
    fn test_parse_func_def() {
        let src = "함수 더하기(가: 정수, 나: 정수) -> 정수 { 반환 가 + 나 }";
        let prog = parse_src(src);
        assert_eq!(prog.stmts.len(), 1);
        match &prog.stmts[0].kind {
            StmtKind::FuncDef {
                name,
                params,
                return_type,
                body,
            } => {
                assert_eq!(name, "더하기");
                assert_eq!(params.len(), 2);
                assert_eq!(params[0], ("가".to_string(), Type::정수));
                assert_eq!(params[1], ("나".to_string(), Type::정수));
                assert!(matches!(return_type, Some(Type::정수)));
                assert_eq!(body.len(), 1);
                assert!(matches!(body[0].kind, StmtKind::Return(Some(_))));
            }
            _ => panic!("Expected FuncDef"),
        }
    }

    #[test]
    fn test_parse_var_decl() {
        let src = "변수 나이 = 20";
        let prog = parse_src(src);
        assert_eq!(prog.stmts.len(), 1);
        match &prog.stmts[0].kind {
            StmtKind::VarDecl {
                name,
                ty,
                value,
                mutable,
            } => {
                assert_eq!(name, "나이");
                assert!(ty.is_none());
                assert!(matches!(value, Expr::IntLiteral(20)));
                assert!(*mutable);
            }
            _ => panic!("Expected VarDecl"),
        }
    }

    #[test]
    fn test_parse_if_else() {
        let src = "만약 x > 0 { 반환 x } 아니면 { 반환 -x }";
        let prog = parse_src(src);
        assert_eq!(prog.stmts.len(), 1);
        match &prog.stmts[0].kind {
            StmtKind::If {
                cond,
                then_block,
                else_block,
            } => {
                assert!(matches!(
                    cond,
                    Expr::BinaryOp {
                        op: BinaryOpKind::Gt,
                        ..
                    }
                ));
                assert_eq!(then_block.len(), 1);
                assert!(else_block.is_some());
                assert_eq!(else_block.as_ref().unwrap().len(), 1);
            }
            _ => panic!("Expected If"),
        }
    }

    #[test]
    fn test_parse_for_loop() {
        let src = "반복 변수 i = 0; i < 10; i += 1 { 출력(i) }";
        let prog = parse_src(src);
        assert_eq!(prog.stmts.len(), 1);
        match &prog.stmts[0].kind {
            StmtKind::ForLoop {
                init,
                cond,
                step: _,
                body,
            } => {
                assert!(matches!(&init.kind, StmtKind::VarDecl { name, .. } if name == "i"));
                assert!(matches!(
                    cond,
                    Expr::BinaryOp {
                        op: BinaryOpKind::Lt,
                        ..
                    }
                ));
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected ForLoop"),
        }
    }

    #[test]
    fn test_expr_precedence() {
        let src = "3 + 5 * 2";
        let prog = parse_src(src);
        assert_eq!(prog.stmts.len(), 1);
        match &prog.stmts[0].kind {
            StmtKind::ExprStmt(Expr::BinaryOp { op, left, right }) => {
                assert_eq!(*op, BinaryOpKind::Add);
                assert!(matches!(left.as_ref(), Expr::IntLiteral(3)));
                match right.as_ref() {
                    Expr::BinaryOp { op, left, right } => {
                        assert_eq!(*op, BinaryOpKind::Mul);
                        assert!(matches!(left.as_ref(), Expr::IntLiteral(5)));
                        assert!(matches!(right.as_ref(), Expr::IntLiteral(2)));
                    }
                    _ => panic!("Expected inner BinaryOp(Mul)"),
                }
            }
            _ => panic!("Expected ExprStmt(BinaryOp(Add, ...))"),
        }
    }

    #[test]
    fn test_parse_while_loop() {
        let src = "동안 참 { 멈춰 }";
        let prog = parse_src(src);
        assert_eq!(prog.stmts.len(), 1);
        match &prog.stmts[0].kind {
            StmtKind::WhileLoop { cond, body } => {
                assert!(matches!(cond, Expr::BoolLiteral(true)));
                assert_eq!(body.len(), 1);
                assert!(matches!(body[0].kind, StmtKind::Break));
            }
            _ => panic!("Expected WhileLoop"),
        }
    }

    #[test]
    fn test_parse_const_decl_with_type() {
        let src = "상수 최대값: 정수 = 100";
        let prog = parse_src(src);
        match &prog.stmts[0].kind {
            StmtKind::VarDecl {
                name,
                ty,
                value,
                mutable,
            } => {
                assert_eq!(name, "최대값");
                assert!(matches!(ty, Some(Type::정수)));
                assert!(matches!(value, Expr::IntLiteral(100)));
                assert!(!mutable);
            }
            _ => panic!("Expected VarDecl"),
        }
    }

    #[test]
    fn test_parse_func_call() {
        let src = "더하기(1, 2)";
        let prog = parse_src(src);
        match &prog.stmts[0].kind {
            StmtKind::ExprStmt(Expr::Call { name, args }) => {
                assert_eq!(name, "더하기");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected ExprStmt(Call)"),
        }
    }

    #[test]
    fn test_parse_else_if_chain() {
        let src = "만약 x > 10 { 출력(1) } 아니면 만약 x > 5 { 출력(2) } 아니면 { 출력(3) }";
        let prog = parse_src(src);
        assert_eq!(prog.stmts.len(), 1);
        match &prog.stmts[0].kind {
            StmtKind::If {
                else_block: Some(else_stmts),
                ..
            } => {
                assert_eq!(else_stmts.len(), 1);
                assert!(matches!(else_stmts[0].kind, StmtKind::If { .. }));
                if let StmtKind::If {
                    else_block: Some(inner_else),
                    ..
                } = &else_stmts[0].kind
                {
                    assert_eq!(inner_else.len(), 1);
                } else {
                    panic!("Expected inner else block");
                }
            }
            _ => panic!("Expected If with else-if chain"),
        }
    }

    #[test]
    fn test_span_tracking() {
        let src = "변수 나이 = 20";
        let prog = parse_src(src);
        assert_eq!(prog.stmts[0].span.line, 1);
    }
}
