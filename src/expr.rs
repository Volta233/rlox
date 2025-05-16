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
        callee: Box<Expr>,      // 被调用的函数表达式
        paren: Token,           // 右括号token
        arguments: Vec<Expr>,    // 参数列表
    },
    Super {
        keyword: Token,      // super关键字token
        method: Token,       // 要调用的方法名
    },
    Assign {
        name: Token,        // 被赋值的目标
        value: Box<Expr>,   // 赋值的表达式
    },
    GetAttribute {
        object: Box<Expr>,  // 对象表达式（如obj.x中的obj）
        name: Token,         // 属性名（如x）
    },
    // GetMethod {
    //     object: Box<Expr>,  // 对象表达式（如obj.x中的obj）
    //     name: Token,         // 类中的方法名（如x）
    // },
    // 属性赋值
    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },
    // this表达式
    This {
        keyword: Token,
    },
}