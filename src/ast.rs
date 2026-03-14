#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

impl Span {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }

    #[cfg(test)]
    pub fn zero() -> Self {
        Self { line: 0, col: 0 }
    }
}

/// Han 언어 타입 시스템
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    정수,   // i64
    실수,   // f64
    문자열, // String / i8*
    불,     // bool / i1
    없음,   // void
}

/// 표현식 AST 노드
#[derive(Debug, Clone)]
pub enum Expr {
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BoolLiteral(bool),
    Identifier(String),
    BinaryOp {
        op: BinaryOpKind,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOpKind,
        expr: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
    Assign {
        name: String,
        value: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOpKind {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOpKind {
    Neg, // -x
    Not, // !x
}

/// 문장 AST — Span 포함 wrapper
#[derive(Debug, Clone)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

impl Stmt {
    pub fn new(kind: StmtKind, span: Span) -> Self {
        Self { kind, span }
    }

    #[cfg(test)]
    pub fn unspanned(kind: StmtKind) -> Self {
        Self {
            kind,
            span: Span::zero(),
        }
    }
}

/// 문장 내부 종류
#[derive(Debug, Clone)]
pub enum StmtKind {
    VarDecl {
        name: String,
        ty: Option<Type>,
        value: Expr,
        #[allow(dead_code)]
        mutable: bool,
    },
    FuncDef {
        name: String,
        params: Vec<(String, Type)>,
        return_type: Option<Type>,
        body: Vec<Stmt>,
    },
    Return(Option<Expr>),
    If {
        cond: Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    },
    ForLoop {
        init: Box<Stmt>,
        cond: Expr,
        step: Box<Stmt>,
        body: Vec<Stmt>,
    },
    WhileLoop {
        cond: Expr,
        body: Vec<Stmt>,
    },
    Break,
    Continue,
    ExprStmt(Expr),
}

/// 프로그램 최상위 노드
#[derive(Debug, Clone)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

impl Program {
    pub fn new(stmts: Vec<Stmt>) -> Self {
        Self { stmts }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_expr() {
        let expr = Expr::BinaryOp {
            op: BinaryOpKind::Add,
            left: Box::new(Expr::IntLiteral(3)),
            right: Box::new(Expr::IntLiteral(5)),
        };
        assert!(matches!(expr, Expr::BinaryOp { .. }));
    }

    #[test]
    fn test_program_new() {
        let prog = Program::new(vec![]);
        assert_eq!(prog.stmts.len(), 0);
    }

    #[test]
    fn test_span() {
        let span = Span::new(10, 5);
        assert_eq!(span.line, 10);
        assert_eq!(span.col, 5);
    }

    #[test]
    fn test_stmt_with_span() {
        let stmt = Stmt::new(StmtKind::ExprStmt(Expr::IntLiteral(42)), Span::new(1, 0));
        assert_eq!(stmt.span.line, 1);
        assert!(matches!(stmt.kind, StmtKind::ExprStmt(_)));
    }
}
