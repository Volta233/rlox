pub mod scanner;
pub mod syntaxer;
pub mod expr;
pub mod statement;
pub mod token;
pub mod interpreter;
pub mod environment;

#[macro_export]
macro_rules! assert_token {
    // 基础类型匹配
    ($scanner:expr, $expected_type:pat) => {{
        let token = $scanner.scan_token();
        assert!(
            matches!(token.token_type, $expected_type),
            "Expected {} but got {:?} at line {}",
            stringify!($expected_type),
            token.token_type,
            token.line
        );
    }};

    // 带字面量检查
    ($scanner:expr, $expected_type:expr, $lexeme:expr, $literal:expr) => {{
        let token = $scanner.scan_token();
        
        assert_eq!(
            token.token_type, $expected_type,
            "Token type mismatch. Expected {:?}, got {:?}",
            $expected_type, token.token_type
        );
        
        assert_eq!(
            token.lexeme, $lexeme,
            "Lexeme mismatch. Expected {:?}, got {:?}",
            $lexeme, token.lexeme
        );

        match ($literal, token.literal.as_ref()) {
            (expected_str, Some(Literal::StringValue(actual_str))) => {
                let expected = expected_str.to_string();
                assert_eq!(
                    actual_str, &expected,
                    "String literal mismatch.\nExpected: {:?}\nGot: {:?}",
                    expected, actual_str
                );
            },
            (expected_num, Some(Literal::NumberValue(actual_num))) => {
                let expected = expected_num as f64;
                assert!(
                    (actual_num - expected).abs() < f64::EPSILON,
                    "Number literal mismatch.\nExpected: {}\nGot: {}",
                    expected, actual_num
                );
            },
            _ => panic!(
                "Literal type mismatch.\nExpected: {:?}\nGot: {:?}",
                $literal, token.literal
            ),
        }
    }};
    
    // 无字面量简化版
    ($scanner:expr, $expected_type:expr, $lexeme:expr) => {
        assert_token!($scanner, $expected_type, $lexeme, None);
    };
}

#[macro_export]
macro_rules! test_error {
    // 错误匹配（现在检查 lexeme 而不是 token_type 中的字符串）
    ($source:expr, $error_msg:expr) => {
        let mut scanner = Scanner::new($source);
        let token = scanner.scan_token();
        if let TokenType::Error = token.token_type {
            assert!(
                token.lexeme.contains($error_msg),
                "Expected error message containing '{}', got '{}'",
                $error_msg,
                token.lexeme
            );
        } else {
            panic!(
                "Expected error token, got {:?} with lexeme '{}'",
                token.token_type,
                token.lexeme
            );
        }
    };
}