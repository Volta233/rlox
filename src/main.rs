use clap::Parser;
use std::fs;
use std::path::Path;
use rlox::scanner::Scanner;

mod token;
mod statement;
mod scanner;
mod syntaxer;
mod expr;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Input Lox file path (.lox)
    #[arg(short, long)]
    input: String,
    
    /// Output directory (optional)
    #[arg(short, long, default_value = "output")]
    output: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // 检查输入文件扩展名
    if !args.input.ends_with(".lox") {
        eprintln!("Error: Input file must have .lox extension");
        std::process::exit(1);
    }

    // 读取Lox文件内容
    let code = fs::read_to_string(&args.input)?;
    let mut scanner = Scanner::new(&code);
    
    // 执行词法分析
    match scanner.scan_tokens() {
        Ok(tokens) => {
            // 新增语法分析
            let mut parser = syntaxer::Parser::new(tokens.clone());
            let ast = match parser.parse() {
                Ok(ast) => ast,
                Err(e) => {
                    eprintln!("Syntax error: {}", e);
                    std::process::exit(1);
                }
            };

            // 输出处理
            if let Some(output_dir) = args.output {
                // 保存词法结果
                let lex_path = Path::new(&output_dir).join("lex_result.json");
                fs::write(lex_path, serde_json::to_string_pretty(&tokens)?)?;
                
                // 保存语法结果
                let ast_path = Path::new(&output_dir).join("ast_result.json");
                fs::write(ast_path, serde_json::to_string_pretty(&ast)?)?;
                println!("Results saved to: {}", output_dir);
            } else {
                println!("AST: {:#?}", ast);
            }
        }
        Err(errors) => {
            // 打印所有词法错误
            eprintln!("Lexical errors found:");
            for err in errors {
                eprintln!("{}", err);
            }
            std::process::exit(1);
        }
    }
    
    Ok(())
}