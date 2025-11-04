use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use clap::Parser;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use favlist_core::{
    export_favlist_blocking,
    ExportOptions,
    ExportProgress,
    ExportResult,
    ProgressCallback,
    FIELDNAMES,
};

#[derive(Parser, Debug)]
#[command(author, version, about = "导出B站收藏夹条目到CSV", long_about = None)]
struct Cli {
    /// 收藏夹页面URL
    fav_url: String,

    /// 输出CSV路径
    #[arg(short = 'o', long = "output", default_value = "favlist.csv")]
    output: PathBuf,

    /// 输出文件编码
    #[arg(short = 'e', long = "encoding", default_value = "utf-8")]
    encoding: String,

    /// 单页请求条目数
    #[arg(long = "page-size", default_value_t = 40)]
    page_size: u32,

    /// 附加Cookie
    #[arg(long = "cookie")]
    cookie: Option<String>,

    /// 请求超时时间（秒）
    #[arg(long = "timeout", default_value_t = 10)]
    timeout: u64,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if cli.page_size == 0 {
        bail!("page-size 必须大于 0");
    }

    println!("{}", style("解析收藏夹链接...").cyan());

    let progress_bar = Arc::new(ProgressBar::new_spinner());
    progress_bar
        .set_style(
            ProgressStyle::with_template("{spinner:.green} {msg}")
                .unwrap_or_else(|_| ProgressStyle::default_spinner()),
        );
    progress_bar.set_message("正在准备抓取收藏夹...");
    progress_bar.enable_steady_tick(Duration::from_millis(120));

    let callback_bar = Arc::clone(&progress_bar);
    let progress_callback: ProgressCallback = Arc::new(move |progress: ExportProgress| {
        if let Some(total) = progress.total {
            callback_bar.set_message(format!("正在抓取收藏夹：已获取 {}/{}", progress.current, total));
        } else {
            callback_bar.set_message(format!("正在抓取收藏夹：已获取 {} 条", progress.current));
        }
    });

    let options = ExportOptions {
        fav_url: cli.fav_url.clone(),
        csv_path: cli.output.clone(),
        encoding: cli.encoding.clone(),
        page_size: cli.page_size,
        cookie: cli.cookie.clone(),
        timeout_secs: cli.timeout,
        timestamp: None,
        extra_headers: Default::default(),
        base_url: None,
        progress_callback: Some(progress_callback),
    };

    let result = export_favlist_blocking(options).with_context(|| "导出收藏夹失败");
    progress_bar.finish_and_clear();
    let result = result?;
    print_summary(&result);
    Ok(())
}

fn print_summary(result: &ExportResult) {
    println!(
        "{} {}",
        style("收藏夹：").green().bold(),
        result.folder_info.title
    );

    match result.total_count {
        Some(total) => println!(
            "{} {}/{}",
            style("处理条目：").cyan(),
            result.processed_count,
            total
        ),
        None => println!(
            "{} {}",
            style("处理条目：").cyan(),
            result.processed_count
        ),
    }

    if result.new_entries.is_empty() {
        println!(
            "{} {}",
            style("新增条目：").yellow(),
            result.new_entries.len()
        );
        println!("{}", style("没有新的条目需要写入。").yellow());
    } else {
        println!(
            "{} {}",
            style("新增条目：").green(),
            result.new_entries.len()
        );
    }

    println!(
        "{} {}",
        style("输出文件：").green(),
        result.csv_path.display()
    );
    println!("{} {}", style("CSV表头：").dim(), FIELDNAMES.join(", "));
}
