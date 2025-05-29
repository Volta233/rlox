use clap::Parser; 
use std::fs;
// use std::path::Path;

use lox::scanner::Scanner;
use lox::syntaxer::Parser as SyntaxParser; // 重命名语法分析器
use lox::interpreter::Interpreter;
use std::error::Error;

#[derive(clap::Parser)] // 明确指定使用 clap 的宏
#[command(author, version, about)]
struct Args {
    // Input Lox file path 
    input: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    // 添加 panic hook 确保错误信息正确格式化
    std::panic::set_hook(Box::new(|panic_info| {
        if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            println!("{}", s);
        } else if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            println!("{}", s);
        }
        std::process::exit(1);
    }));

    let args = Args::parse();

    let code = fs::read_to_string(&args.input)?;
    let mut scanner = Scanner::new(&code);
    
    let tokens = scanner.scan_tokens().map_err(|errs| {
        let first_err = errs.first().unwrap();
        println!("{}", first_err);
        std::process::exit(1);
    })?;

    // 保存词法分析结果
    // let lex_path = Path::new("output").join("lex_result.json");
    // fs::write(lex_path, serde_json::to_string_pretty(&tokens)?)?;
    // println!("[DEBUG] finish lexeme scanner.");

    // 语法分析错误处理
    let mut parser = SyntaxParser::new(tokens);
    let ast = parser.parse().map_err(|e| {
        // 使用 Display 格式输出错误
        println!("{}", e);
        std::process::exit(1);
    })?;

    // 保存语法树
    // let ast_path = Path::new("output").join("ast_result.json");
    // fs::write(ast_path, serde_json::to_string_pretty(&ast)?)?;
    // println!("[DEBUG] finish parser.");

    // 解释执行错误处理
    let mut my_interpreter = Interpreter::new();
    my_interpreter.interpret(&ast).map_err(|e| {
        // 使用 Display 格式输出错误
        println!("{}", e);
        std::process::exit(1);
    })?;

    // println!("[DEBUG] finish interpreter.");
    Ok(())
}