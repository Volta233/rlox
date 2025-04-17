pub mod scanner;  // 导出scanner模块
pub mod syntaxer;
pub mod expr;
pub mod statement;
pub mod token;    // 导出token模块

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

    // 带字面量检查（处理不同类型）
    ($scanner:expr, $expected_type:expr, $lexeme:expr, $literal:expr) => {{
        let token = $scanner.scan_token();
        
        // 类型匹配
        assert_eq!(
            token.token_type, $expected_type,
            "Token type mismatch. Expected {:?}, got {:?}",
            $expected_type, token.token_type
        );
        
        // 原始字符串匹配
        assert_eq!(
            token.lexeme, $lexeme,
            "Lexeme mismatch. Expected {:?}, got {:?}",
            $lexeme, 
            token.lexeme
        );

        // 字面量特殊处理逻辑
        match ($literal, token.literal.as_ref()) {
            // 处理字符串比较
            (expected_str, Some(Literal::StringValue(actual_str))) => {
                let expected = expected_str.to_string();
                assert_eq!(
                    actual_str, &expected,
                    "String literal mismatch.\nExpected: {:?}\nGot: {:?}",
                    expected, actual_str
                );
            }
            
            // 处理数字比较
            (expected_num, Some(Literal::NumberValue(actual_num))) => {
                let expected = expected_num as f64;
                assert!(
                    (actual_num - expected).abs() < f64::EPSILON,
                    "Number literal mismatch.\nExpected: {}\nGot: {}",
                    expected, actual_num
                );
            }
            
            // // 处理布尔值比较
            // (expected_bool, Some(Literal::Boolean(actual_bool))) => {
            //     assert_eq!(
            //         actual_bool, 
            //         expected_bool,
            //         "Boolean literal mismatch.\nExpected: {}\nGot: {}",
            //         expected_bool, 
            //         actual_bool
            //     );
            // }
            
            // 其他类型错误
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
    // 错误匹配
    ($source:expr, $error_msg:expr) => {
        let token = scan_token($source);
        if let TokenType::Error(msg) = token.token_type {
            assert!(msg.contains($error_msg));
        } else {
            panic!("Expected error token");
        }
    };
}