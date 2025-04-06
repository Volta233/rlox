use clap::Parser;
use std::fs;
use std::path::Path;

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
    
    // 读取输入文件
    let code = fs::read_to_string(&args.input)?;
    
    // 创建词法分析器
    let mut scanner = Scanner::new(&code);
    let tokens = scanner.scan_tokens();
    
    // 生成输出路径
    let output_path = Path::new(&args.output)
        .join("lex_result.json");
    
    // 序列化输出
    let json = serde_json::to_string_pretty(&tokens)?;
    fs::write(output_path, json)?;
    
    Ok(())
}