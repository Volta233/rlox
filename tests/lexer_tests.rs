use crate::scanner::Scanner; // 直接通过crate根路径导入
use crate::token::{Token, TokenType}; 
use crate::assert_token; // 导入自定义宏

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
    let code = "var fun class if else while for return";
    let mut scanner = Scanner::new(code);
    
    assert_token!(scanner, TokenType::KeywordVar);
    assert_token!(scanner, TokenType::KeywordFun);
    assert_token!(scanner, TokenType::KeywordClass);
    assert_token!(scanner, TokenType::KeywordIf);
    assert_token!(scanner, TokenType::KeywordElse);
    assert_token!(scanner, TokenType::KeywordWhile);
    assert_token!(scanner, TokenType::KeywordFor);
    assert_token!(scanner, TokenType::KeywordReturn);
}

#[test]
fn test_number_literals() {
    let code = "123 456.789 .5"; // 测试整数、小数、前导小数点
    let mut scanner = Scanner::new(code);
    
    if let TokenType::Number(n) = scanner.scan_token().token_type {
        assert!((n - 123.0).abs() < f64::EPSILON);
    } else { panic!("Not number token"); }
    
    if let TokenType::Number(n) = scanner.scan_token().token_type {
        assert!((n - 456.789).abs() < f64::EPSILON);
    } else { panic!("Not number token"); }
    
    // 测试非法数字格式
    let token = scanner.scan_token();
    assert!(matches!(token.token_type, TokenType::Error(_)));
}

#[test]
fn test_string_literals() {
    let code = r#""hello" "world\n" "#;
    let mut scanner = Scanner::new(code);
    
    if let TokenType::String(s) = scanner.scan_token().token_type {
        assert_eq!(s, "hello");
    } else { panic!("Not string token"); }
    
    if let TokenType::String(s) = scanner.scan_token().token_type {
        assert_eq!(s, "world\n");
    } else { panic!("Not string token"); }
}

#[test]
fn test_error_recovery() {
    let code = "@# invalid tokens";
    let mut scanner = Scanner::new(code);
    
    let token = scanner.scan_token();
    assert!(matches!(token.token_type, TokenType::Error(_)));
    
    // 验证错误后能继续解析
    let token = scanner.scan_token();
    assert!(matches!(token.token_type, TokenType::Error(_)));
}