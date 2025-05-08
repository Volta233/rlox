use serde::Serialize;
use std::collections::HashMap;
use crate::statement::Stmt;
use crate::environment::Environment;

#[derive(Debug, Clone, Serialize)]
pub struct LoxFunction {
    pub declaration: Box<Stmt>, // 使用Box包装语句
    pub closure: Box<Environment>, // 使用Box包装环境
}

#[derive(Debug, Clone, Serialize)]
pub struct LoxClass {
    pub name: String,
    pub methods: Vec<Stmt>,
    pub superclass: Option<Box<LoxClass>>,
    pub closure: Box<Environment>, // 新增closure字段
}

impl LoxClass {
    pub fn find_method(&self, name: &str) -> Option<Literal> {
        // 在当前类查找
        if let Some(method) = self.methods.iter().find(|m| match m {
            Stmt::Function { name: token, .. } => token.lexeme == name,
            _ => false
        }) {
            return Some(Literal::FunctionValue(LoxFunction {
                declaration: Box::new(method.clone()),
                closure: self.closure.clone(), // 使用类的closure字段
            }));
        }

        // 递归查找超类
        if let Some(superclass) = &self.superclass {
            superclass.find_method(name)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LoxInstance {
    pub class: LoxClass,
    pub fields: HashMap<String, Literal>,
}

#[derive(Debug, Clone, Serialize)]
pub enum Literal {
    StringValue(String),
    NumberValue(f64),
    Boolean(bool),
    Nil,
    FunctionValue(LoxFunction),
    ClassValue(LoxClass),
    InstanceValue(LoxInstance),
    None,
}

impl Literal {
    pub fn as_class(&self) -> Option<LoxClass> {
        if let Literal::ClassValue(c) = self {
            Some(c.clone())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TokenType {
    // --- 单字符符号 ---
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // --- 一或两个字符符号 ---
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // --- 字面量 ---
    Identifier,
    String,
    Number,

    // --- 关键字 ---
    And,    
    Class,
    Else,
    False,  
    Fun,
    For,
    If,
    Nil,    
    Or,    
    Print,  
    Return,
    Super,  
    This,   
    True,   
    Var,
    While,

    // --- 错误类型 ---
    Error,

    // --- 其他 ---
    Eof,
}

#[derive(Debug, Clone, Serialize)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub lexeme: String,
    pub literal: Option<Literal>,
}

impl Token {
    pub fn new(
        token_type: TokenType, 
        line: usize, 
        lexeme: String,
        literal: Option<Literal>
    ) -> Self {
        Self {
            token_type,
            line,
            lexeme,
            literal,
        }
    }

    pub fn this() -> Self {
        Self {
            token_type: TokenType::This,
            line: 0,
            lexeme: "this".into(),
            literal: None,
        }
    }
}