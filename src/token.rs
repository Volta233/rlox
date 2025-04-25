use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum Literal {
    StringValue(String),
    NumberValue(f64),
    Boolean(bool),
    Nil,
    None,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TokenType {
    // --- 单字符符号（Single-character tokens） ---
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

    // --- 一或两个字符符号（One or two character tokens） ---
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // --- 字面量（Literals）---
    Identifier,  // 移除 String 参数，用 lexeme 和 literal 分离
    String,
    Number,

    // --- 关键字（Keywords）---
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

    // --- 其他 ---
    Error,
    Eof,
}

#[derive(Debug, Clone, Serialize)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub lexeme: String,     // 原始字符串内容
    pub literal: Option<Literal>,   // 字面量的值
}

impl Token {
    pub fn new(
        token_type: TokenType, 
        line: usize, 
        lexeme: String,
        literal: Option<Literal>  // 新增 literal 参数
    ) -> Self {
        Self {
            token_type,
            line,
            lexeme,
            literal,             // 保存字面量
        }
    }
}