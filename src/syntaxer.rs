use crate::token::{Token, TokenType, Literal};
use crate::expr::Expr;
use crate::statement::Stmt;
use std::fmt;
use std::error::Error;

// ------------------- 错误处理结构 -------------------
#[derive(Debug)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[line {}] Syntax Error: {}", self.line, self.message)
    }
}

impl Error for ParseError {}  // 实现 Error trait

// ------------------- 语法分析器主体 -------------------
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { 
            tokens,
            current: 0 
        }
    }

    // 主解析方法
    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    // --------------- 声明解析 ---------------
    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        let result = if self.match_token(TokenType::Class) {
            self.class_declaration()
        } else if self.match_token(TokenType::Fun) {
            self.function("function")
        } else if self.match_token(TokenType::Var) {
            self.var_declaration()
        } else {
            self.statement()
        };

        result.or_else(|err| {
            self.synchronize();
            Err(err)
        })
    }

    // --------------- 类声明 ---------------
    fn class_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume_identifier("Expect class name")?;

        let superclass = if self.match_token(TokenType::Less) {
            let super_token = self.consume_identifier("Expect superclass name")?;
            Some(Expr::Variable { name: super_token })
        } else {
            None
        };

        self.consume(TokenType::LeftBrace, "Expect '{' before class body")?;

        let mut methods = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            methods.push(self.function("method")?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after class body")?;
        Ok(Stmt::Class { name, superclass, methods })
    }

    // --------------- 函数声明 ---------------
    fn function(&mut self, kind: &str) -> Result<Stmt, ParseError> {
        let name = self.consume_identifier(&format!("Expect {} name", kind))?;
        self.consume(TokenType::LeftParen, &format!("Expect '(' after {} name", kind))?;

        let mut params = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                if params.len() >= 255 {
                    return Err(self.error(&self.peek(), "Can't have more than 255 parameters"));
                }
                params.push(self.consume_identifier("Expect parameter name")?);
                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expect ')' after parameters")?;
        self.consume(TokenType::LeftBrace, &format!("Expect '{{' before {} body", kind))?;
        
        let body = self.block_statement()?;
        Ok(Stmt::Function { name, params, body })
    }

    // --------------- 变量声明 ---------------
    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume_identifier("Expect variable name")?;

        let initializer = if self.match_token(TokenType::Equal) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "Expect ';' after variable declaration")?;
        Ok(Stmt::VarDecl { name, initializer })
    }

    // --------------- 语句解析 ---------------
    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_token(TokenType::For) {
            self.for_statement()
        } else if self.match_token(TokenType::If) {
            self.if_statement()
        } else if self.match_token(TokenType::Print) {
            self.print_statement()
        } else if self.match_token(TokenType::Return) {
            self.return_statement()
        } else if self.match_token(TokenType::While) {
            self.while_statement()
        } else if self.match_token(TokenType::LeftBrace) {
            Ok(Stmt::Block { statements: self.block_statement()? })
        } else {
            self.expression_statement()
        }
    }

    // --------------- for 语句 ---------------
    fn for_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'")?;

        let initializer = if self.match_token(TokenType::Semicolon) {
            None
        } else if self.match_token(TokenType::Var) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if !self.check(TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::Semicolon, "Expect ';' after loop condition")?;

        let increment = if !self.check(TokenType::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::RightParen, "Expect ')' after for clauses")?;

        let mut body = self.statement()?;

        if let Some(inc) = increment {
            body = Stmt::Block {
                statements: vec![body, Stmt::Expression { expression: inc }],
            };
        }

        let while_loop = Stmt::While {
            condition: condition.unwrap_or(Expr::Literal {
                value: Literal::Boolean(true),
            }),
            body: Box::new(body),
        };

        Ok(if let Some(init) = initializer {
            Stmt::Block { statements: vec![init, while_loop] }
        } else {
            while_loop
        })
    }

    // --------------- 代码块 ---------------
    fn block_statement(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block")?;
        Ok(statements)
    }

    // --------------- if 语句 ---------------
    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition")?;

        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_token(TokenType::Else) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If { condition, then_branch, else_branch })
    }

    // --------------- 表达式语句 ---------------
    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression")?;
        Ok(Stmt::Expression { expression: expr })
    }

    // --------------- print 语句 ---------------
    fn print_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value")?;
        Ok(Stmt::Print { expression: value })
    }

    // --------------- return 语句 ---------------
    fn return_statement(&mut self) -> Result<Stmt, ParseError> {
        let keyword = self.previous().clone();
        let value = if !self.check(TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::Semicolon, "Expect ';' after return value")?;
        Ok(Stmt::Return { keyword, value })
    }

    // --------------- while 语句 ---------------
    fn while_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition")?;
        let body = Box::new(self.statement()?);
        Ok(Stmt::While { condition, body })
    }

    // --------------- 表达式解析 ---------------
    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.equality()?;
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        while self.match_tokens(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;

        while self.match_tokens(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;

        while self.match_tokens(&[TokenType::Plus, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;

        while self.match_tokens(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_tokens(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(TokenType::False) {
            Ok(Expr::Literal { value: Literal::Boolean(false) })
        } else if self.match_token(TokenType::True) {
            Ok(Expr::Literal { value: Literal::Boolean(true) })
        } else if self.match_token(TokenType::Nil) {
            Ok(Expr::Literal { value: Literal::Nil })
        } else if self.match_token(TokenType::Number) {
            Ok(Expr::Literal { value: self.previous().literal.clone().unwrap() })
        } else if self.match_token(TokenType::String) {
            Ok(Expr::Literal { value: self.previous().literal.clone().unwrap() })
        } else if self.match_token(TokenType::LeftParen) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression")?;
            Ok(Expr::Grouping { expression: Box::new(expr) })
        } else {
            Err(self.error(&self.peek(), "Expect expression"))
        }
    }

    // --------------- 工具方法 ---------------
    fn match_token(&mut self, ttype: TokenType) -> bool {
        if self.check(ttype) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn match_tokens(&mut self, ttypes: &[TokenType]) -> bool {
        ttypes.iter().any(|t| self.check(t.clone()))
    }

    fn consume(&mut self, ttype: TokenType, message: &str) -> Result<(), ParseError> {
        if self.check(ttype) {
            self.advance();
            Ok(())
        } else {
            Err(self.error(&self.peek(), message))
        }
    }

    fn consume_identifier(&mut self, msg: &str) -> Result<Token, ParseError> {
        if self.check(TokenType::Identifier) {
            Ok(self.peek().clone())
        } else {
            Err(self.error(&self.peek(), msg))
        }
    }

    fn error(&self, token: &Token, message: &str) -> ParseError {
        ParseError {
            line: token.line,
            message: format!("{} (found '{}')", message, token.lexeme),
        }
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }
            match self.peek().token_type {
                TokenType::Class | TokenType::Fun | TokenType::Var
                | TokenType::For | TokenType::If | TokenType::While
                | TokenType::Print | TokenType::Return => return,
                _ => self.advance(),
            }
        }
    }

    fn check(&self, ttype: TokenType) -> bool {
        !self.is_at_end() && self.peek().token_type == ttype
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.current += 1;
        }
    }

    fn previous(&self) -> &Token {
        &self.tokens[(self.current - 1).max(0)]
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }
}