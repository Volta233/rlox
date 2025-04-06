#[derive(Debug, Clone)]
pub enum TokenType {
    // 单字符Token
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,
    
    // 单/双字符Token  
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,
    
    // 字面量
    Identifier(String),
    String(String),
    Number(f64),
    
    // 关键字
    KeywordVar, KeywordFun, KeywordClass, 
    KeywordIf, KeywordElse, KeywordWhile,
    KeywordFor, KeywordReturn,
    
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub lexeme: String,  // 新增字段保存原始字符串
}

impl Token {
    pub fn new(token_type: TokenType, line: usize, lexeme: String) -> Self {
        Self { token_type, line, lexeme }
    }
}