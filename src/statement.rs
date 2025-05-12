use crate::token::Token;
use serde::Serialize;
use crate::expr::Expr;

#[derive(Clone, Debug, Serialize)]
pub enum Stmt {
    Block {
        statements: Vec<Stmt>,
    },
    Expression {
        expression: Expr,
    },
    VarDecl {
        name: Token,
        initializer: Option<Expr>,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
    Class {
        name: Token,
        superclass: Option<Expr>,
        methods: Vec<Stmt>,
    },
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    For {
        initializer: Option<Box<Stmt>>,
        condition: Option<Expr>,
        increment: Option<Expr>,
        body: Box<Stmt>,
    },
    Return {
        keyword: Token,
        value: Option<Expr>,
    },
    Print {
        expression: Expr,
    },
}