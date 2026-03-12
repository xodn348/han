#![allow(dead_code, unused)]

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
    // 리터럴
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BoolLiteral(bool),
    // 식별자
    Identifier(String),
    // 이항 연산: left op right
    BinaryOp {
        op: BinaryOpKind,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    // 단항 연산: op expr
    UnaryOp {
        op: UnaryOpKind,
        expr: Box<Expr>,
    },
    // 함수 호출: name(args)
    Call {
        name: String,
        args: Vec<Expr>,
    },
    // 할당: name = value
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

/// 문장 AST 노드
#[derive(Debug, Clone)]
pub enum Stmt {
    // 변수 선언: 변수 name: type = value
    VarDecl {
        name: String,
        ty: Option<Type>,
        value: Expr,
        mutable: bool,
    },
    // 함수 정의: 함수 name(params) -> return_type { body }
    FuncDef {
        name: String,
        params: Vec<(String, Type)>,
        return_type: Option<Type>,
        body: Vec<Stmt>,
    },
    // 반환문: 반환 value
    Return(Option<Expr>),
    // 조건문: 만약 cond { then } 아니면 { else }
    If {
        cond: Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    },
    // for 반복문: 반복 init; cond; step { body }
    ForLoop {
        init: Box<Stmt>,
        cond: Expr,
        step: Box<Stmt>,
        body: Vec<Stmt>,
    },
    // while 반복문: 동안 cond { body }
    WhileLoop {
        cond: Expr,
        body: Vec<Stmt>,
    },
    // 멈춰
    Break,
    // 계속
    Continue,
    // 표현식 문장
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
}
