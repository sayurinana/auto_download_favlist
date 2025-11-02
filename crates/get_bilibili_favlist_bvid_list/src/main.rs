use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::Parser;
use console::style;
use favlist_core::{export_favlist_blocking, ExportOptions, ExportResult, FIELDNAMES};

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
    let options = ExportOptions {
        fav_url: cli.fav_url.clone(),
        csv_path: cli.output.clone(),
        encoding: cli.encoding.clone(),
        page_size: cli.page_size,
        cookie: cli.cookie.clone(),
        timeout_secs: cli.timeout,
        timestamp: None,
        extra_headers: Default::default(),
    };

    let result = export_favlist_blocking(options).with_context(|| "导出收藏夹失败")?;
    print_summary(&result);
    Ok(())
}

fn print_summary(result: &ExportResult) {
    println!(
        "{} {}",
        style("收藏夹：").green().bold(),
        result.folder_info.title
    );
    if result.new_entries.is_empty() {
        println!("{}", style("没有新的条目需要写入。").yellow());
    } else {
        println!(
            "{} {} 条记录，输出文件：{}",
            style("写入完成，新增加").green(),
            result.new_entries.len(),
            result.csv_path.display()
        );
    }
    println!("{} {}", style("CSV表头包含：").dim(), FIELDNAMES.join(", "));
}
