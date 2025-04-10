use rlox::scanner::Scanner; // 直接通过crate根路径导入
use rlox::token::{TokenType, Literal}; 
use rlox::assert_token; // 导入自定义宏

#[test]
fn test_basic_symbols() {
    let code = "(){},.-+;*/";
    let mut scanner = Scanner::new(code);
    
    let expected = vec![
        TokenType::LeftParen,
        TokenType::RightParen,
        TokenType::LeftBrace,
        TokenType::RightBrace,
        TokenType::Comma,
        TokenType::Dot,
        TokenType::Minus,
        TokenType::Plus,
        TokenType::Semicolon,
        TokenType::Star,
        TokenType::Slash,
        TokenType::Eof,
    ];
    
    for token_type in expected {
        let token = scanner.scan_token();
        assert_eq!(token.token_type, token_type);
    }
}

#[test]
fn test_keywords() {
    let code = "var fun class if else while for return or true";
    let mut scanner = Scanner::new(code);
    
    assert_token!(scanner, TokenType::Var);
    assert_token!(scanner, TokenType::Fun);
    assert_token!(scanner, TokenType::Class);
    assert_token!(scanner, TokenType::If);
    assert_token!(scanner, TokenType::Else);
    assert_token!(scanner, TokenType::While);
    assert_token!(scanner, TokenType::For);
    assert_token!(scanner, TokenType::Return);
    assert_token!(scanner, TokenType::Or);
    assert_token!(scanner, TokenType::True);
}

#[test]
fn test_number_literals() {
    let code = "123 456.789 .5";
    let mut scanner = Scanner::new(code);
    
    assert_token!(scanner, TokenType::Number, "123", 123.0);
    assert_token!(scanner, TokenType::Number, "456.789", 456.789);
}

