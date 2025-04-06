pub mod scanner;  // 导出scanner模块
pub mod token;    // 导出token模块

#[macro_export]
macro_rules! assert_token {
    // 基础匹配模式（参考网页3的宏结构）
    ($scanner:expr, $expected:pat) => {{
        // 获取下一个token并解构（参考网页7的Token处理）
        let token = $scanner.scan_token();
        
        // 使用stringify!生成友好错误信息（参考网页5的stringify应用）
        assert!(
            matches!(token.token_type, $expected),
            "Expected {} but got {:?} at line {}",
            stringify!($expected),  // 将模式转为字符串
            token.token_type,
            token.line
        );
    }};
    
    // 带值匹配的扩展模式（处理带参数的TokenType）
    ($scanner:expr, $expected:pat, $value:expr) => {{
        let token = $scanner.scan_token();
        assert!(
            matches!(token.token_type, $expected),
            "Expected {} with value {} but got {:?} at line {}",
            stringify!($expected),
            $value,
            token.token_type,
            token.line
        );
        // 验证具体值（如数字字面量）
        if let $expected = token.token_type {
            assert_eq!($value, actual_value);
        }
    }};
}