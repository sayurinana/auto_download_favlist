use std::io::{self, Write};

use anyhow::{Context, Result};

pub fn prompt_input(message: &str, default: Option<&str>) -> Result<String> {
    print_prompt(message, default)?;
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).context("读取输入失败")?;
    let input = buffer.trim().to_string();
    if input.is_empty() {
        if let Some(default) = default {
            Ok(default.to_string())
        } else {
            Ok(String::new())
        }
    } else {
        Ok(input)
    }
}

fn print_prompt(message: &str, default: Option<&str>) -> Result<()> {
    let mut stdout = io::stdout();
    match default {
        Some(value) if !value.is_empty() => {
            write!(stdout, "{} [{}]: ", message, value)?;
        }
        _ => {
            write!(stdout, "{}: ", message)?;
        }
    }
    stdout.flush().context("刷新提示失败")?;
    Ok(())
}

pub fn pause_with_message(message: &str) -> Result<()> {
    println!("{}", message);
    let mut stdout = io::stdout();
    stdout.flush().ok();
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).ok();
    Ok(())
}
