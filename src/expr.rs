use crate::token::{Token, Literal};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: Literal,
    },
    Variable {
        name: Token,
    },
    Call {
        callee: Box<Expr>,      // 被调用的函数表达式（如函数名）
        paren: Token,           // 右括号token（用于错误定位）
        arguments: Vec<Expr>,    // 参数列表
    },
    Super {
        keyword: Token,      // super关键字token
        method: Token,       // 要调用的方法名
    },
}