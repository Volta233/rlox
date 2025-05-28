use std::{
    path::Path,
    process::Command,
    time::Instant,
    fs,
};
use colored::Colorize;
use std::process::Stdio; 

fn main() {
    let cases_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap()
        .join("tests\\cases");

    // 检查目录存在性
    if !cases_path.exists() {
        eprintln!("{} 测试目录不存在: {}", "[错误]".red().bold(), cases_path.display());
        std::process::exit(1);
    }

    // 串行执行测试
    let mut passed = 0;
    for case_id in 1..=35 {
        let (_id, is_pass, msg) = run_single_test(case_id, &cases_path);
        println!("{}", msg);
        if is_pass { passed += 1; }
    }

    // 最终统计
    println!("\n{} 总用例: {}  通过: {}  失败: {}",
        "结果汇总:".cyan().bold(),
        35.to_string().yellow(),
        passed.to_string().green(),
        (35-passed).to_string().red()
    );
}

fn run_single_test(case_id: usize, base_path: &Path) -> (usize, bool, String) {
    let in_file = base_path.join(format!("{}.in", case_id));
    let out_file = base_path.join(format!("{}.out", case_id));

    // 文件检查
    if !in_file.exists() || !out_file.exists() {
        return (case_id, false, format!(
            "[Case {:02}] {} → {}",
            case_id,
            in_file.display().to_string().bright_blue(),
            "[SKIP] 文件缺失".yellow()
        ));
    }

    // 执行测试
    let start = Instant::now();
    let output = match execute_with_timeout(&in_file) {
        Ok(o) => o,
        Err(e) => return (case_id, false, format!(
            "[Case {:02}] {} → {} ({:.2}s)\n{}",
            case_id,
            in_file.display().to_string().bright_blue(),
            "[ERROR]".red(),
            start.elapsed().as_secs_f64(),
            e
        ))
    };

    // 结果比对
    let expected = fs::read_to_string(out_file).unwrap_or_default();

    // 标准化换行符为 \n
    let process_output = |s: &str| -> Vec<String> {
        s.replace("\r\n", "\n")       // 统一换行符
        .split('\n')                // 按行分割
        .map(|line| line.trim())     // 处理每行首尾空格
        .filter(|s| !s.is_empty())   // 过滤空行（按需调整）
        .map(String::from)
        .collect::<Vec<_>>()
    };

    let expected_lines = process_output(&expected);
    let actual_lines = process_output(&output);
    let passed = expected_lines == actual_lines;
    
    // 生成报告
    let status = if passed {
        format!("[PASS] {}", "✓".green())
    } else {
        format!("[FAIL] {}", "✗".red())
    };

    let msg = if passed {
        format!(
            "[Case {:02}] {} → {} ({:.2}s)",
            case_id,
            in_file.display().to_string().bright_blue(),
            status,
            start.elapsed().as_secs_f64()
        )
    } else {
        format!(
            "[Case {:02}] {} → {} ({:.2}s)\n{}{}\n{}{}",
            case_id,
            in_file.display().to_string().bright_blue(),
            status,
            start.elapsed().as_secs_f64(),
            "预期: ".yellow(),
            expected.trim(),
            "实际: ".yellow(),
            output.trim()
        )
    };

    (case_id, passed, msg)
}

fn execute_with_timeout(input_path: &Path) -> Result<String, String> {
    // 获取项目根目录
    let root_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap();
    
    // 构建正确解释器路径
    let interpreter = root_dir.join("target\\release\\lox.exe");
    
    let mut cmd = Command::new(interpreter);
    cmd.arg(input_path);

    // 显式重定向输出流
    cmd.stdout(Stdio::piped())
       .stderr(Stdio::piped());

    let output = cmd.output()
        .map_err(|e| e.to_string())?; 

    // 合并输出流
    let mut combined = String::new();
    if !output.stdout.is_empty() {
        combined.push_str(&String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        combined.push_str(&String::from_utf8_lossy(&output.stderr));
    }

    Ok(combined.trim().to_string())
}