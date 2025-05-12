use clap::Parser; 
use std::fs;
use std::path::Path;
use rlox::scanner::Scanner;
use rlox::syntaxer::Parser as SyntaxParser; // 重命名语法分析器
use rlox::interpreter::Interpreter;
use std::error::Error;

#[derive(clap::Parser)] // 明确指定使用 clap 的宏
#[command(author, version, about)]
struct Args {
    // Input Lox file path 
    input: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let code = fs::read_to_string(&args.input)?;
    let mut scanner = Scanner::new(&code);
    
    match scanner.scan_tokens() {
        Ok(tokens) => {
            // 使用语法分析器
            let tokens_clone = tokens.clone();

            let lex_path = Path::new("output").join("lex_result.json");
            fs::write(lex_path, serde_json::to_string_pretty(&tokens_clone)?)?;

            let mut parser = SyntaxParser::new(tokens);
            let ast = match parser.parse() {
                Ok(ast) => ast,
                Err(e) => return Err(Box::new(e)), // 自动转换到 dyn Error
            };
            
            let ast_path = Path::new("output").join("ast_result.json");
            fs::write(ast_path, serde_json::to_string_pretty(&ast)?)?;
            let mut my_interpreter = Interpreter::new();
            my_interpreter.interpret(&ast)
                    .map_err(|e| Box::new(e) as Box<dyn Error>)?;
            Ok(())  // 整个函数成功返回
        }
        Err(errors) => {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                errors.join("\n")
            )))
        }
    }
}