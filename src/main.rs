use clap::Parser; // 这是 clap 的 Parser trait
use std::fs;
use std::path::Path;
use rlox::scanner::Scanner;
use rlox::syntaxer::Parser as SyntaxParser; // 重命名语法分析器


#[derive(clap::Parser)] // 明确指定使用 clap 的宏
#[command(author, version, about)]
struct Args {
    /// Input Lox file path 
    #[arg(short, long)]
    input: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let code = fs::read_to_string(&args.input)?;
    let mut scanner = Scanner::new(&code);
    
    match scanner.scan_tokens() {
        Ok(tokens) => {
            // 使用重命名后的语法分析器
            let tokens_clone = tokens.clone();
            let mut parser = SyntaxParser::new(tokens);
            let ast = match parser.parse() {
                Ok(ast) => ast,
                Err(e) => return Err(Box::new(e)), // 自动转换到 dyn Error
            };

            let lex_path = Path::new("output").join("lex_result.json");
            fs::write(lex_path, serde_json::to_string_pretty(&tokens_clone)?)?;
            
            let ast_path = Path::new("output").join("ast_result.json");
            fs::write(ast_path, serde_json::to_string_pretty(&ast)?)?;
            println!("Debug output saved to output/");
            Ok(())
        }
        Err(errors) => {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                errors.join("\n")
            )))
        }
    }
}