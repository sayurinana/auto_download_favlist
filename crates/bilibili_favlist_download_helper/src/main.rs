mod bbdown;
mod config;
mod menu;
mod prompts;

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::terminal;
use favlist_core::inventory::{
    diff_new_entries, find_missing_videos, scan_directory_bvids, write_inventory_file,
};
use favlist_core::{
    current_timestamp, export_favlist_blocking, read_csv_rows, CsvRow, ExportOptions,
};

use bbdown::run_bbdown;
use config::{ConfigStore, FavConfig};
use menu::{select_from_menu, MenuOutcome};
use prompts::{pause_with_message, prompt_input};

#[derive(Parser, Debug)]
#[command(author, version, about = "B 站收藏夹下载助手", long_about = None)]
struct Cli {
    /// 自定义配置文件路径
    #[arg(long = "config-path")]
    config_path: Option<PathBuf>,

    /// Dry-run 模式，仅打印将执行的命令
    #[arg(long = "dry-run")]
    dry_run: bool,
}

struct App {
    store: ConfigStore,
    dry_run: bool,
}

impl App {
    fn new(store: ConfigStore, dry_run: bool) -> Self {
        Self { store, dry_run }
    }

    fn run(&mut self) -> Result<()> {
        loop {
            let action = self.main_menu()?;
            match action {
                MainAction::NewConfig => self.handle_new_config()?,
                MainAction::UseConfig => self.handle_existing_configs()?,
                MainAction::Exit => {
                    println!("已退出助手。");
                    break;
                }
            }
        }
        Ok(())
    }

    fn main_menu(&mut self) -> Result<MainAction> {
        let options = vec![
            "录入新收藏夹".to_string(),
            "使用存档配置".to_string(),
            "退出程序".to_string(),
        ];
        match select_from_menu("请选择操作", &options)? {
            MenuOutcome::Selected(0) => Ok(MainAction::NewConfig),
            MenuOutcome::Selected(1) => Ok(MainAction::UseConfig),
            MenuOutcome::Selected(_) | MenuOutcome::Esc => Ok(MainAction::Exit),
        }
    }

    fn handle_new_config(&mut self) -> Result<()> {
        terminal::disable_raw_mode().ok();
        println!("录入新收藏夹（留空可取消）");

        let fav_url = prompt_input("请输入收藏夹 URL", None)?;
        if fav_url.is_empty() {
            println!("已取消录入。");
            pause_with_message("按回车返回菜单...")?;
            terminal::enable_raw_mode().ok();
            return Ok(());
        }

        let default_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .display()
            .to_string();
        let download_dir_input = prompt_input("请输入下载目录", Some(&default_dir))?;
        let download_dir = PathBuf::from(download_dir_input);
        fs::create_dir_all(&download_dir)
            .with_context(|| format!("创建目录失败: {}", download_dir.display()))?;

        let encoding = prompt_input("CSV 编码(默认 utf-8)", Some("utf-8"))?;
        let encoding = if encoding.is_empty() {
            "utf-8".to_string()
        } else {
            encoding
        };
        let page_size = prompt_input("分页大小(默认40)", Some("40"))?;
        let page_size = page_size.parse::<u32>().unwrap_or(40).max(1);
        let timeout_secs = prompt_input("请求超时(秒, 默认10)", Some("10"))?;
        let timeout_secs = timeout_secs.parse::<u64>().unwrap_or(10).max(1);
        let cookie = prompt_input("Cookie(可留空)", None)?;
        let cookie = if cookie.is_empty() {
            None
        } else {
            Some(cookie)
        };
        let name = prompt_input("配置名称(可留空)", None)?;
        let name = if name.is_empty() { None } else { Some(name) };

        let timestamp = current_timestamp();
        let csv_path = download_dir.join(format!("{timestamp}-favlist.csv"));

        let options = ExportOptions {
            fav_url: fav_url.clone(),
            csv_path: csv_path.clone(),
            encoding: encoding.clone(),
            page_size,
            cookie: cookie.clone(),
            timeout_secs,
            timestamp: Some(timestamp.clone()),
            extra_headers: Default::default(),
            base_url: None,
        };

        println!("正在抓取收藏夹，请稍候...");
        match export_favlist_blocking(options) {
            Ok(result) => {
                println!(
                    "抓取完成，共新增 {} 条记录，输出文件：{}",
                    result.new_entries.len(),
                    result.csv_path.display()
                );
                let config = FavConfig {
                    fav_url,
                    download_dir: download_dir.display().to_string(),
                    csv_path: result.csv_path.display().to_string(),
                    encoding,
                    page_size,
                    cookie,
                    timeout_secs,
                    last_synced_at: Some(result.timestamp),
                    name,
                };
                self.store.add(config)?;
            }
            Err(err) => {
                println!("抓取失败: {err}");
            }
        }

        pause_with_message("按回车返回菜单...")?;
        terminal::enable_raw_mode().ok();
        Ok(())
    }

    fn handle_existing_configs(&mut self) -> Result<()> {
        if self.store.configs().is_empty() {
            terminal::disable_raw_mode().ok();
            println!("当前没有任何已保存的配置。可先录入新收藏夹。");
            pause_with_message("按回车返回菜单...")?;
            terminal::enable_raw_mode().ok();
            return Ok(());
        }

        let options: Vec<String> = self
            .store
            .configs()
            .iter()
            .enumerate()
            .map(|(idx, cfg)| {
                let title = cfg
                    .name
                    .as_ref()
                    .filter(|name| !name.is_empty())
                    .cloned()
                    .unwrap_or_else(|| format!("收藏夹 {}", idx + 1));
                format!("{} -> {}", title, cfg.fav_url)
            })
            .collect();

        match select_from_menu("选择配置", &options)? {
            MenuOutcome::Selected(index) => self.handle_config_actions(index)?,
            MenuOutcome::Esc => {}
        }
        Ok(())
    }

    fn handle_config_actions(&mut self, index: usize) -> Result<()> {
        loop {
            let options = vec![
                "编辑配置".to_string(),
                "检查更新".to_string(),
                "检查缺漏".to_string(),
                "返回".to_string(),
            ];
            match select_from_menu("配置操作", &options)? {
                MenuOutcome::Selected(0) => self.edit_config(index)?,
                MenuOutcome::Selected(1) => self.check_update(index)?,
                MenuOutcome::Selected(2) => self.check_missing(index)?,
                MenuOutcome::Selected(_) | MenuOutcome::Esc => break,
            }
        }
        Ok(())
    }

    fn edit_config(&mut self, index: usize) -> Result<()> {
        let mut config = self.store.configs()[index].clone();
        terminal::disable_raw_mode().ok();
        println!("编辑配置（留空保持原值，输入 '-' 删除可选字段）");

        let download_dir = prompt_input("下载目录", Some(&config.download_dir))?;
        if !download_dir.is_empty() {
            fs::create_dir_all(&download_dir)
                .with_context(|| format!("创建目录失败: {download_dir}"))?;
            config.download_dir = download_dir;
        }

        let fav_url = prompt_input("收藏夹 URL", Some(&config.fav_url))?;
        if !fav_url.is_empty() {
            config.fav_url = fav_url;
        }

        let csv_path = prompt_input("CSV 路径", Some(&config.csv_path))?;
        if !csv_path.is_empty() {
            config.csv_path = csv_path;
        }

        let encoding = prompt_input("编码", Some(&config.encoding))?;
        if !encoding.is_empty() {
            config.encoding = encoding;
        }

        let page_size = prompt_input("分页大小", Some(&config.page_size.to_string()))?;
        if !page_size.is_empty() {
            config.page_size = page_size.parse::<u32>().unwrap_or(config.page_size).max(1);
        }

        let timeout_secs = prompt_input("超时(秒)", Some(&config.timeout_secs.to_string()))?;
        if !timeout_secs.is_empty() {
            config.timeout_secs = timeout_secs
                .parse::<u64>()
                .unwrap_or(config.timeout_secs)
                .max(1);
        }

        let cookie = prompt_input("Cookie (- 表示清除)", config.cookie.as_deref())?;
        if cookie == "-" {
            config.cookie = None;
        } else if !cookie.is_empty() {
            config.cookie = Some(cookie);
        }

        let name = prompt_input("展示名称 (- 表示清除)", config.name.as_deref())?;
        if name == "-" {
            config.name = None;
        } else if !name.is_empty() {
            config.name = Some(name);
        }

        self.store.update(index, config)?;

        pause_with_message("配置已更新，按回车返回...")?;
        terminal::enable_raw_mode().ok();
        Ok(())
    }

    fn check_update(&mut self, index: usize) -> Result<()> {
        let mut config = self.store.configs()[index].clone();
        terminal::disable_raw_mode().ok();
        println!("检查更新...");

        let old_csv_path = config.csv_path();
        let old_rows = read_csv_rows(&old_csv_path, &config.encoding).unwrap_or_default();
        let backup_path = old_csv_path.with_extension("backup.csv");
        let had_old_file = old_csv_path.exists();
        if had_old_file {
            fs::rename(&old_csv_path, &backup_path)
                .with_context(|| format!("备份旧 CSV 失败: {}", backup_path.display()))?;
            println!("旧 CSV 已备份至 {}", backup_path.display());
        }

        let timestamp = current_timestamp();
        let new_csv_path = config
            .download_dir_path()
            .join(format!("{timestamp}-favlist.csv"));

        let options = ExportOptions {
            fav_url: config.fav_url.clone(),
            csv_path: new_csv_path.clone(),
            encoding: config.encoding.clone(),
            page_size: config.page_size,
            cookie: config.cookie.clone(),
            timeout_secs: config.timeout_secs,
            timestamp: Some(timestamp.clone()),
            extra_headers: Default::default(),
            base_url: None,
        };

        match export_favlist_blocking(options) {
            Ok(result) => {
                let new_rows = read_csv_rows(&new_csv_path, &config.encoding)?;
                let diffs = diff_new_entries(&old_rows, &new_rows);
                if diffs.is_empty() {
                    println!("未发现新增条目。");
                } else {
                    println!("发现 {} 个新增条目：", diffs.len());
                    let download_dir = config.download_dir_path();
                    fs::create_dir_all(&download_dir)?;
                    let mut count = 0;
                    for row in diffs {
                        if let Some(bvid) = extract_bvid(&row) {
                            println!("下载 {}", bvid);
                            if let Err(err) = run_bbdown(&bvid, &download_dir, self.dry_run) {
                                println!("bbdown 失败: {err}");
                            }
                            count += 1;
                        }
                    }
                    println!("新增条目处理完成，总计 {} 个", count);
                }
                config.csv_path = new_csv_path.display().to_string();
                config.last_synced_at = Some(result.timestamp);
                self.store.update(index, config)?;
            }
            Err(err) => {
                println!("导出失败: {err}");
                if had_old_file {
                    fs::rename(&backup_path, &old_csv_path).with_context(|| "恢复旧 CSV 失败")?;
                }
            }
        }

        pause_with_message("按回车返回...")?;
        terminal::enable_raw_mode().ok();
        Ok(())
    }

    fn check_missing(&mut self, index: usize) -> Result<()> {
        let config = self.store.configs()[index].clone();
        terminal::disable_raw_mode().ok();
        println!("检查缺漏...");

        let download_dir = config.download_dir_path();
        if !download_dir.exists() {
            println!("下载目录不存在: {}", download_dir.display());
            pause_with_message("按回车返回...")?;
            terminal::enable_raw_mode().ok();
            return Ok(());
        }

        let mapping = match scan_directory_bvids(&download_dir) {
            Ok(data) => data,
            Err(err) => {
                println!("扫描下载目录失败: {err}");
                pause_with_message("按回车返回...")?;
                terminal::enable_raw_mode().ok();
                return Ok(());
            }
        };

        match write_inventory_file(&download_dir, &mapping) {
            Ok(path) => println!("已生成目录清单：{}", path.display()),
            Err(err) => println!("生成目录清单失败: {err}"),
        }

        let csv_rows = read_csv_rows(&config.csv_path(), &config.encoding)?;
        let existing_bvids: Vec<String> = mapping.keys().cloned().collect();
        let missing_rows = find_missing_videos(&csv_rows, &existing_bvids);
        if missing_rows.is_empty() {
            println!("未检测到缺失的视频。");
        } else {
            println!("检测到 {} 个缺失条目：", missing_rows.len());
            for row in missing_rows {
                if let Some(bvid) = extract_bvid(&row) {
                    println!("补全 {}", bvid);
                    if let Err(err) = run_bbdown(&bvid, &download_dir, self.dry_run) {
                        println!("bbdown 失败: {err}");
                    }
                }
            }
        }

        pause_with_message("按回车返回...")?;
        terminal::enable_raw_mode().ok();
        Ok(())
    }
}

fn extract_bvid(row: &CsvRow) -> Option<String> {
    for key in ["bv_id", "BV号", "视频BV号"] {
        if let Some(value) = row.get(key) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

enum MainAction {
    NewConfig,
    UseConfig,
    Exit,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let store = ConfigStore::load(cli.config_path.clone())?;
    let mut app = App::new(store, cli.dry_run);
    app.run()
}
