use std::collections::HashMap;
use crate::token::{Token, TokenType, Literal};

pub struct Scanner {
    source: Vec<char>,
    current: usize,    // 当前扫描位置（绝对索引）
    start: usize,     // 当前词素起始位置
    line: usize,      // 当前行号
    had_error: bool,  // 错误状态标记（新增）
}

impl Scanner {
    /// 初始化扫描器
    pub fn new(source: &str) -> Self {
        let mut keywords = HashMap::new();
        // 初始化所有保留字（新增 AND/OR/PRINT 等）
        keywords.insert("and", TokenType::And);
        keywords.insert("class", TokenType::Class);
        keywords.insert("else", TokenType::Else);
        keywords.insert("false", TokenType::False);
        keywords.insert("for", TokenType::For);
        keywords.insert("fun", TokenType::Fun);
        keywords.insert("if", TokenType::If);
        keywords.insert("nil", TokenType::Nil);
        keywords.insert("or", TokenType::Or);
        keywords.insert("print", TokenType::Print);
        keywords.insert("return", TokenType::Return);
        keywords.insert("super", TokenType::Super);
        keywords.insert("this", TokenType::This);
        keywords.insert("true", TokenType::True);
        keywords.insert("var", TokenType::Var);
        keywords.insert("while", TokenType::While);

        Self {
            source: source.chars().collect(),
            current: 0,
            start: 0,
            line: 1,
            had_error: false,
        }
    }

    /// 核心扫描方法（返回 Result 处理错误）
    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, Vec<String>> {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();

        loop {
            let token = self.scan_token();
            if let TokenType::Error = token.token_type {
                errors.push(token.lexeme.clone());  // 从 lexeme 获取错误信息
                self.had_error = true;
            }
            let is_eof = matches!(token.token_type, TokenType::Eof);
            tokens.push(token);
            if is_eof { break; }
        }

        if errors.is_empty() {
            Ok(tokens)
        } else {
            Err(errors)
        }
    }

    /// 扫描单个 token
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
            
            // 双字符操作符（新增错误位置信息）
            '!' => self.make_dual_char_token('=', TokenType::BangEqual, TokenType::Bang),
            '=' => self.make_dual_char_token('=', TokenType::EqualEqual, TokenType::Equal),
            '<' => self.make_dual_char_token('=', TokenType::LessEqual, TokenType::Less),
            '>' => self.make_dual_char_token('=', TokenType::GreaterEqual, TokenType::Greater),
            
            // 字符串字面量（新增转义字符处理）
            '"' => self.scan_string(),
            
            // 数字字面量（修改为存储 literal）
            c if c.is_ascii_digit() => self.scan_number(),

            // 标识符/关键字（使用新关键字映射）
            c if c.is_ascii_alphabetic() || c == '_' => self.scan_identifier(),

            // 未识别字符（增强错误信息）
            _ => self.error_token(&format!("Unexpected character '{}'", c)),
        }
    }

    /// 扫描字符串字面量（新增转义处理）
    fn scan_string(&mut self) -> Token {
        let mut value = String::new();
        let mut error = None;

        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            let c = self.advance();
            if c == '\\' {
                // 处理转义字符
                match self.advance() {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    '"' => value.push('"'),
                    '\\' => value.push('\\'),
                    esc => error = Some(format!("Invalid escape sequence \\{}", esc)),
                }
            } else {
                value.push(c);
            }
        }

        if self.is_at_end() {
            return self.error_token("Unterminated string");
        }

        self.advance(); // 消耗闭合引号

        if let Some(err) = error {
            self.error_token(&err)
        } else {
            self.make_token_with_literal(TokenType::String, Literal::StringValue(value))
        }
    }

    /// 扫描数字字面量（存储为 Literal）
    fn scan_number(&mut self) -> Token {
        let mut is_float = false;
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            is_float = true;
            self.advance(); // 消耗小数点
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let num_str: String = self.source[self.start..self.current].iter().collect();
        match num_str.parse() {
            Ok(num) => self.make_token_with_literal(
                TokenType::Number,
                if is_float { Literal::NumberValue(num) } else { Literal::NumberValue(num as f64) }
            ),
            Err(_) => self.error_token(&format!("Invalid number {}", num_str)),
        }
    }

    /// 统一标识符扫描方法（更名并优化关键字查找）
    fn scan_identifier(&mut self) -> Token {
        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
            self.advance();
        }
        
        let text: String = self.source[self.start..self.current]
            .iter()
            .collect();
        
        // 通过模式匹配优化关键字查找
        let token_type = match text.as_str() {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fun" => TokenType::Fun,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            _ => TokenType::Identifier, // 注意这里改为无参数形式
        };
        
        self.make_token(token_type)
    }
    /// 创建带字面量的 token（新增方法）
    fn make_token_with_literal(&self, token_type: TokenType, literal: Literal) -> Token {
        let lexeme = self.source[self.start..self.current].iter().collect();
        Token::new(token_type, self.line, lexeme, Some(literal))
    }

    /// 处理双字符操作符（核心逻辑）
    fn make_dual_char_token(
        &mut self,
        expected: char,
        matched_type: TokenType,
        unmatched_type: TokenType
    ) -> Token {
        if self.match_char(expected) {
            self.make_token(matched_type)
        } else {
            self.make_token(unmatched_type)
        }
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
                '/' if self.peek_next() == '/' => {
                    // 处理单行注释
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
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
        Token::new(
            token_type, 
            self.line, 
            lexeme,
            None
        )
    }

    /// 带错误信息的 token（新增行号）
    fn error_token(&mut self, message: &str) -> Token {
        self.had_error = true;
        Token::new(
            TokenType::Error,  // 使用简单的 Error 枚举值
            self.line,
            format!("[line {}] {}", self.line, message),  // 错误信息放在 lexeme
            None
        )
    }
    /// 检查是否到达输入结尾
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}