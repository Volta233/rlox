use std::collections::HashMap;
use crate::token::{Token, TokenType};

pub struct Scanner {
    source: Vec<char>,
    current: usize,    // 当前扫描位置
    start: usize,     // 当前token起始位置
    line: usize,      // 当前行号
    keywords: HashMap<&'static str, TokenType>, // 关键字映射表
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        let mut keywords = HashMap::new();
        // 初始化关键字映射
        keywords.insert("var", TokenType::KeywordVar);
        keywords.insert("fun", TokenType::KeywordFun);
        keywords.insert("class", TokenType::KeywordClass);
        keywords.insert("if", TokenType::KeywordIf);
        keywords.insert("else", TokenType::KeywordElse);
        keywords.insert("while", TokenType::KeywordWhile);
        keywords.insert("for", TokenType::KeywordFor);
        keywords.insert("return", TokenType::KeywordReturn);

        Self {
            source: source.chars().collect(),
            current: 0,
            start: 0,
            line: 1,
            keywords,
        }
    }

    /// 核心扫描方法
    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenType::Eof);
        }

        let c = self.advance();

        match c {
            // 单字符token
            '(' => self.make_token(TokenType::LeftParen),
            ')' => self.make_token(TokenType::RightParen),
            '{' => self.make_token(TokenType::LeftBrace),
            '}' => self.make_token(TokenType::RightBrace),
            ',' => self.make_token(TokenType::Comma),
            '.' => self.make_token(TokenType::Dot),
            '-' => self.make_token(TokenType::Minus),
            '+' => self.make_token(TokenType::Plus),
            ';' => self.make_token(TokenType::Semicolon),
            '*' => self.make_token(TokenType::Star),
            '/' => {
                if self.match_char('/') {
                    // 处理单行注释
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                    self.scan_token() // 递归调用跳过注释
                } else {
                    self.make_token(TokenType::Slash)
                }
            }
            
            // 双字符操作符（参考网页7的匹配逻辑）
            '!' => self.make_token_if('=', TokenType::BangEqual, TokenType::Bang),
            '=' => self.make_token_if('=', TokenType::EqualEqual, TokenType::Equal),
            '<' => self.make_token_if('=', TokenType::LessEqual, TokenType::Less),
            '>' => self.make_token_if('=', TokenType::GreaterEqual, TokenType::Greater),
            
            // 字符串字面量（此处省略，需处理转义字符）
            '"' => self.string(),
            
            // 数字字面量
            c if c.is_ascii_digit() => self.number(),
            
            // 标识符/关键字
            c if c.is_ascii_alphabetic() || c == '_' => self.identifier(),
            
            _ => self.error_token("Unexpected character."),
        }
    }

    fn make_token_if(&mut self, expected: char, matched_type: TokenType, unmatched_type: TokenType) -> Token {
        if self.match_char(expected) {
            self.make_token(matched_type)
        } else {
            self.make_token(unmatched_type)
        }
    }

    fn string(&mut self) -> Token {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return self.error_token("Unterminated string");
        }

        self.advance(); // 消耗闭合的引号
        let value: String = self.source[self.start + 1..self.current - 1]
            .iter()
            .collect();
        
        self.make_token(TokenType::String(value))
    }
    /// 处理标识符和关键字
    fn identifier(&mut self) -> Token {
        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
            self.advance();
        }
        
        let text: String = self.source[self.start..self.current]
            .iter()
            .collect();
        
        // 查找关键字
        let token_type = self.keywords
            .get(text.as_str())
            .map(|t| t.clone())  
            .unwrap_or(TokenType::Identifier(text));
        
        self.make_token(token_type)
    }

    /// 处理数字字面量
    fn number(&mut self) -> Token {
        // 整数部分
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        // 小数部分
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance(); // 消耗小数点
            
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let num_str: String = self.source[self.start..self.current]
            .iter()
            .collect();
        let num = num_str.parse::<f64>().unwrap();
        self.make_token(TokenType::Number(num))
    }

    /// 移动指针并返回当前字符
    fn advance(&mut self) -> char {
        self.current += 1;
        self.source.get(self.current - 1).copied().unwrap_or('\0')
    }

    /// 跳过空白字符
    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                _ => break,
            }
        }
    }

    /// 查看下一个字符
    fn peek(&self) -> char {
        self.source.get(self.current).copied().unwrap_or('\0')
    }

    /// 查看下下个字符
    fn peek_next(&self) -> char {
        self.source.get(self.current + 1).copied().unwrap_or('\0')
    }

    /// 条件匹配字符
    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source[self.current] != expected {
            return false;
        }
        self.current += 1;
        true
    }

    
    fn make_token(&self, token_type: TokenType) -> Token {
        let lexeme: String = self.source[self.start..self.current]
            .iter()
            .collect();
        Token::new(token_type, self.line, lexeme)
    }

    fn error_token(&self, message: &str) -> Token {
        Token::new(
            TokenType::Error(message.to_string()),
            self.line,
            String::new(),
        )
    }
    /// 检查是否到达输入结尾
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.scan_token();
            let is_eof = matches!(token.token_type, TokenType::Eof);
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        tokens
    }
}