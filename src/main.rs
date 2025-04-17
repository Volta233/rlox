use clap::Parser;
use std::fs;
use std::path::Path;
use std::io::{self, Write};
use rlox::scanner::Scanner;

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
            // 如果有输出目录，保存结果到JSON文件
            if let Some(output_dir) = args.output {
                let output_path = Path::new(&output_dir).join("lex_result.json");
                fs::create_dir_all(&output_dir)?;
                let json = serde_json::to_string_pretty(&tokens)?;
                fs::write(output_path, json)?;
                println!("Lexical analysis results saved to: {}", output_dir);
            } 
            // 否则直接打印结果到终端
            else {
                println!("{:#?}", tokens);
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