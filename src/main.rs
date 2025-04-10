use clap::Parser;
use std::fs;
use std::path::Path;
use rlox::scanner::Scanner;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Input Lox file path
    #[arg(short, long)]
    input: String,
    
    /// Output directory
    #[arg(short, long, default_value = "output")]
    output: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let code = fs::read_to_string(&args.input)?;
    
    let mut scanner = Scanner::new(&code);
    let tokens = scanner.scan_tokens(); // 使用新方法
    
    let output_path = Path::new(&args.output).join("lex_result.json");
    let json = serde_json::to_string_pretty(&tokens)?;
    fs::write(output_path, json)?;
    
    Ok(())
}