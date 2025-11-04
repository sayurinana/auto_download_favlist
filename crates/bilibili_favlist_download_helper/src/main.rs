mod bbdown;
mod config;
mod menu;
mod prompts;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use console::style;
use crossterm::terminal;
use favlist_core::inventory::{
    diff_new_entries, find_missing_videos, scan_directory_bvids, write_inventory_file,
};
use favlist_core::{
    current_timestamp, export_favlist_blocking, read_csv_rows, CsvRow, ExportOptions,
    ExportProgress, ProgressCallback,
};
use indicatif::{ProgressBar, ProgressStyle};

use bbdown::{run_bbdown, start_bbdown_serve, BbdownApiClient};
use config::{
    ConfigStore, FavConfig, GlobalDefaultsStore, DEFAULT_BBDOWN_URL, DEFAULT_POLL_INTERVAL_MS,
};
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
    defaults: GlobalDefaultsStore,
    dry_run: bool,
}

impl App {
    fn new(store: ConfigStore, defaults: GlobalDefaultsStore, dry_run: bool) -> Self {
        Self {
            store,
            defaults,
            dry_run,
        }
    }

    fn run(&mut self) -> Result<()> {
        loop {
            let action = self.main_menu()?;
            match action {
                MainAction::NewConfig => self.handle_new_config()?,
                MainAction::UseConfig => self.handle_existing_configs()?,
                MainAction::EditDefaults => self.handle_global_defaults()?,
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
            "设置全局默认".to_string(),
            "退出程序".to_string(),
        ];
        match select_from_menu("请选择操作", &options)? {
            MenuOutcome::Selected(0) => Ok(MainAction::NewConfig),
            MenuOutcome::Selected(1) => Ok(MainAction::UseConfig),
            MenuOutcome::Selected(2) => Ok(MainAction::EditDefaults),
            MenuOutcome::Selected(_) | MenuOutcome::Esc => Ok(MainAction::Exit),
        }
    }

    fn handle_new_config(&mut self) -> Result<()> {
        terminal::disable_raw_mode().ok();
        println!("录入新收藏夹（留空可取消）");

        let fav_url = prompt_input("请输入收藏夹 URL", None)?;
        if fav_url.trim().is_empty() {
            println!("已取消录入。");
            pause_with_message("按回车返回菜单...")?;
            terminal::enable_raw_mode().ok();
            return Ok(());
        }

        let defaults_snapshot = self.defaults.data().clone();
        let env_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .display()
            .to_string();

        let api_prompt_default = defaults_snapshot
            .api_download_dir
            .as_deref()
            .unwrap_or(&env_dir);
        let api_download_dir = resolve_string_with_default(
            prompt_input("BBDown 下载目录(Windows)", Some(api_prompt_default))?,
            defaults_snapshot.api_download_dir.as_ref(),
            api_prompt_default,
        );

        let scan_prompt_default = defaults_snapshot
            .scan_download_dir
            .as_deref()
            .unwrap_or(&env_dir);
        let scan_download_dir = resolve_string_with_default(
            prompt_input("本地检测目录(WSL)", Some(scan_prompt_default))?,
            defaults_snapshot.scan_download_dir.as_ref(),
            scan_prompt_default,
        );
        fs::create_dir_all(PathBuf::from(&scan_download_dir))
            .with_context(|| format!("创建目录失败: {scan_download_dir}"))?;

        let encoding_input = prompt_input("CSV 编码(默认 utf-8)", Some("utf-8"))?;
        let encoding = if encoding_input.trim().is_empty() {
            "utf-8".to_string()
        } else {
            encoding_input.trim().to_string()
        };
        let page_size_input = prompt_input("分页大小(默认40)", Some("40"))?;
        let page_size = page_size_input.trim().parse::<u32>().unwrap_or(40).max(1);
        let timeout_input = prompt_input("请求超时(秒, 默认10)", Some("10"))?;
        let timeout_secs = timeout_input.trim().parse::<u64>().unwrap_or(10).max(1);
        let cookie = normalize_optional(prompt_input("Cookie(可留空)", None)?);
        let name = normalize_optional(prompt_input("配置名称(可留空)", None)?);

        let file_pattern = resolve_optional_with_default(
            prompt_input(
                "BBDown File Pattern(可留空沿用 bbdown.config)",
                defaults_snapshot.file_pattern.as_deref(),
            )?,
            defaults_snapshot.file_pattern.as_ref(),
        );
        let multi_file_pattern = resolve_optional_with_default(
            prompt_input(
                "BBDown Multi File Pattern(可留空)",
                defaults_snapshot.multi_file_pattern.as_deref(),
            )?,
            defaults_snapshot.multi_file_pattern.as_ref(),
        );
        let serve_url_input = prompt_input(
            "BBDown serve 地址(留空使用默认)",
            defaults_snapshot
                .bbdown_serve_url
                .as_deref()
                .or(Some(DEFAULT_BBDOWN_URL)),
        )?;
        let bbdown_serve_url = resolve_string_with_default(
            serve_url_input,
            defaults_snapshot.bbdown_serve_url.as_ref(),
            DEFAULT_BBDOWN_URL,
        );
        let auto_launch_input = prompt_input("缺漏补全时自动启动BBDown serve? (Y/n)", Some("Y"))?;
        let bbdown_auto_launch = parse_bool_input(&auto_launch_input, true);
        let launch_args_input =
            prompt_input("bbdown serve 启动附加参数(使用空格分隔，可留空)", None)?;
        let bbdown_launch_args = if launch_args_input.trim().is_empty() {
            Vec::new()
        } else {
            parse_args(&launch_args_input)
        };
        let default_poll = DEFAULT_POLL_INTERVAL_MS.to_string();
        let poll_interval_input =
            prompt_input("任务状态轮询间隔毫秒(默认 500)", Some(&default_poll))?;
        let bbdown_poll_interval_ms = poll_interval_input
            .trim()
            .parse::<u64>()
            .unwrap_or(DEFAULT_POLL_INTERVAL_MS)
            .max(50);

        let timestamp = current_timestamp();
        let csv_path = Path::new(&scan_download_dir).join(format!("{timestamp}-favlist.csv"));

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
            progress_callback: None,
        };

        match self.run_export_with_progress(options, "抓取收藏夹进度") {
            Ok(result) => {
                println!(
                    "抓取完成，共新增 {} 条记录，输出文件：{}",
                    style(result.new_entries.len()).green(),
                    result.csv_path.display()
                );
                let mut config = FavConfig {
                    fav_url,
                    api_download_dir,
                    scan_download_dir: Some(scan_download_dir.clone()),
                    csv_path: result.csv_path.display().to_string(),
                    encoding,
                    page_size,
                    cookie,
                    timeout_secs,
                    last_synced_at: Some(result.timestamp),
                    name,
                    bbdown_serve_url,
                    bbdown_auto_launch,
                    bbdown_launch_args,
                    bbdown_poll_interval_ms,
                    file_pattern,
                    multi_file_pattern,
                };
                config.apply_defaults();
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

    fn run_export_with_progress(
        &self,
        mut options: ExportOptions,
        label: &str,
    ) -> Result<favlist_core::ExportResult> {
        let display_label = label.to_string();
        let progress_bar = ProgressBar::new_spinner();
        progress_bar.set_style(
            ProgressStyle::with_template("{spinner:.green} {msg}")
                .unwrap_or_else(|_| ProgressStyle::default_spinner()),
        );
        progress_bar.set_message(display_label.clone());
        progress_bar.enable_steady_tick(Duration::from_millis(120));
        let spinner = progress_bar.clone();
        let label_clone = display_label.clone();
        options.progress_callback = Some(Arc::new(move |progress: ExportProgress| {
            if let Some(total) = progress.total {
                spinner.set_message(format!(
                    "{}：已获取 {}/{}",
                    label_clone, progress.current, total
                ));
            } else {
                spinner.set_message(format!("{}：已获取 {} 条", label_clone, progress.current));
            }
        }) as ProgressCallback);
        let result = export_favlist_blocking(options).map_err(|err| err.into());
        progress_bar.finish_and_clear();
        result
    }

    fn handle_global_defaults(&mut self) -> Result<()> {
        terminal::disable_raw_mode().ok();
        println!("设置全局默认（留空保留原值，输入 '-' 清除）");

        let snapshot = self.defaults.data().clone();
        let api_input = prompt_input(
            "默认 BBDown 下载目录(Windows)",
            snapshot.api_download_dir.as_deref(),
        )?;
        let scan_input = prompt_input(
            "默认 本地检测目录(WSL)",
            snapshot.scan_download_dir.as_deref(),
        )?;
        let serve_input = prompt_input(
            "默认 BBDown serve 地址",
            snapshot
                .bbdown_serve_url
                .as_deref()
                .or(Some(DEFAULT_BBDOWN_URL)),
        )?;
        let file_pattern_input =
            prompt_input("默认 File Pattern", snapshot.file_pattern.as_deref())?;
        let multi_file_pattern_input = prompt_input(
            "默认 Multi File Pattern",
            snapshot.multi_file_pattern.as_deref(),
        )?;

        {
            let data = self.defaults.data_mut();
            data.api_download_dir = match api_input.trim() {
                "" => snapshot.api_download_dir,
                "-" => None,
                other => Some(other.to_string()),
            };
            data.scan_download_dir = match scan_input.trim() {
                "" => snapshot.scan_download_dir,
                "-" => None,
                other => Some(other.to_string()),
            };
            data.bbdown_serve_url = match serve_input.trim() {
                "" => snapshot
                    .bbdown_serve_url
                    .or_else(|| Some(DEFAULT_BBDOWN_URL.to_string())),
                "-" => None,
                other => Some(other.to_string()),
            };
            data.file_pattern = match file_pattern_input.trim() {
                "" => snapshot.file_pattern,
                "-" => None,
                other => Some(other.to_string()),
            };
            data.multi_file_pattern = match multi_file_pattern_input.trim() {
                "" => snapshot.multi_file_pattern,
                "-" => None,
                other => Some(other.to_string()),
            };
        }

        self.defaults.save()?;
        println!("{}", style("全局默认已更新。").green());
        pause_with_message("按回车返回菜单...")?;
        terminal::enable_raw_mode().ok();
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

        let download_dir = prompt_input("下载目录(Windows)", Some(&config.api_download_dir))?;
        if !download_dir.is_empty() {
            config.api_download_dir = download_dir;
        }

        let scan_dir = prompt_input("本地检测目录(WSL)", config.scan_download_dir.as_deref())?;
        if !scan_dir.is_empty() {
            fs::create_dir_all(Path::new(&scan_dir))
                .with_context(|| format!("创建目录失败: {scan_dir}"))?;
            config.scan_download_dir = Some(scan_dir);
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

        let serve_url = prompt_input("BBDown serve 地址", Some(&config.bbdown_serve_url))?;
        if !serve_url.trim().is_empty() {
            config.bbdown_serve_url = serve_url.trim().to_string();
        }

        let auto_launch_input = prompt_input(
            "自动启动BBDown serve? (Y/n)",
            Some(if config.bbdown_auto_launch { "Y" } else { "n" }),
        )?;
        config.bbdown_auto_launch = parse_bool_input(&auto_launch_input, config.bbdown_auto_launch);

        let current_args_string = if config.bbdown_launch_args.is_empty() {
            None
        } else {
            Some(config.bbdown_launch_args.join(" "))
        };
        let launch_args_input = prompt_input(
            "serve 附加参数(空格分隔, '-' 清除)",
            current_args_string.as_deref(),
        )?;
        if launch_args_input == "-" {
            config.bbdown_launch_args.clear();
        } else if !launch_args_input.trim().is_empty() {
            config.bbdown_launch_args = parse_args(&launch_args_input);
        }

        let poll_interval_input = prompt_input(
            "任务轮询间隔(毫秒)",
            Some(&config.bbdown_poll_interval_ms.to_string()),
        )?;
        if !poll_interval_input.trim().is_empty() {
            config.bbdown_poll_interval_ms = poll_interval_input
                .trim()
                .parse::<u64>()
                .unwrap_or(config.bbdown_poll_interval_ms)
                .max(50);
        }

        let file_pattern =
            prompt_input("File Pattern (- 表示清除)", config.file_pattern.as_deref())?;
        if file_pattern == "-" {
            config.file_pattern = None;
        } else if !file_pattern.trim().is_empty() {
            config.file_pattern = Some(file_pattern.trim().to_string());
        }

        let multi_file_pattern = prompt_input(
            "Multi File Pattern (- 表示清除)",
            config.multi_file_pattern.as_deref(),
        )?;
        if multi_file_pattern == "-" {
            config.multi_file_pattern = None;
        } else if !multi_file_pattern.trim().is_empty() {
            config.multi_file_pattern = Some(multi_file_pattern.trim().to_string());
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
            .scan_download_dir_path()
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
            progress_callback: None,
        };

        match export_favlist_blocking(options) {
            Ok(result) => {
                let new_rows = read_csv_rows(&new_csv_path, &config.encoding)?;
                let diffs = diff_new_entries(&old_rows, &new_rows);
                if diffs.is_empty() {
                    println!("未发现新增条目。");
                } else {
                    println!("发现 {} 个新增条目：", diffs.len());
                    let download_dir = config.scan_download_dir_path();
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

        let download_dir = config.scan_download_dir_path();
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
            println!("{}", style("未检测到缺失的视频。").green());
        } else {
            println!(
                "{}",
                style(format!("检测到 {} 个缺失条目：", missing_rows.len())).yellow()
            );
            for row in &missing_rows {
                if let Some(bvid) = extract_bvid(row) {
                    println!("• {}", style(bvid).cyan());
                }
            }

            let missing_bvids: Vec<String> = missing_rows.iter().filter_map(extract_bvid).collect();
            let file_pattern = config.resolve_file_pattern();
            let multi_file_pattern = config.resolve_multi_file_pattern();
            let serve_url = config.effective_serve_url().to_string();

            if missing_bvids.is_empty() {
                println!("{}", style("缺失列表中未找到有效的 BV 号。").red());
            } else if self.dry_run {
                println!(
                    "{}",
                    style("当前为 dry-run 模式，将仅展示拟提交的任务与配置。").yellow()
                );
                println!("目标服务：{}", serve_url);
                if let Some(pattern) = &file_pattern {
                    println!("FilePattern: {}", pattern);
                }
                if let Some(pattern) = &multi_file_pattern {
                    println!("MultiFilePattern: {}", pattern);
                }
                for bvid in &missing_bvids {
                    println!(
                        "[dry-run] POST {}/add-task {{ Url: \"{}\" }}",
                        serve_url, bvid
                    );
                }
                println!(
                    "{}",
                    style("未执行实际下载操作，目录状态保持不变。").yellow()
                );
            } else {
                let mut serve_process = None;
                if config.bbdown_auto_launch {
                    match start_bbdown_serve(&config.bbdown_launch_args) {
                        Ok(process) => {
                            println!("{}", style("已启动 bbdown serve 子进程。").green());
                            serve_process = Some(process);
                        }
                        Err(err) => {
                            println!(
                                "{}",
                                style(format!(
                                    "自动启动 bbdown serve 失败：{err}。将尝试连接已有服务。"
                                ))
                                .yellow()
                            );
                        }
                    }
                } else {
                    println!(
                        "{}",
                        style(format!("使用外部 bbdown serve 服务：{}", serve_url)).cyan()
                    );
                }

                let api = BbdownApiClient::new(&serve_url, Duration::from_secs(30))?;
                for bvid in &missing_bvids {
                    api.add_task(bvid, file_pattern.as_deref(), multi_file_pattern.as_deref())
                        .with_context(|| format!("提交下载任务 {bvid} 失败"))?;
                }

                let progress_bar = ProgressBar::new_spinner();
                progress_bar.set_style(
                    ProgressStyle::with_template("{spinner:.green} {msg}")
                        .unwrap_or_else(|_| ProgressStyle::default_spinner()),
                );
                progress_bar.enable_steady_tick(Duration::from_millis(120));
                let spinner = progress_bar.clone();
                api.wait_until_idle(config.poll_interval(), move |running| {
                    if let Some(task) = running.first() {
                        let title = task.title.as_deref().unwrap_or("未命名任务");
                        spinner.set_message(format!(
                            "等待下载完成，剩余 {} 个任务（{}）",
                            running.len(),
                            title
                        ));
                    } else {
                        spinner.set_message("等待下载任务完成...");
                    }
                })?;
                progress_bar.finish_with_message("下载任务已全部完成");

                if let Err(err) = api.remove_finished() {
                    println!("移除已完成任务时出现问题：{err}");
                }
                drop(serve_process);

                println!("正在重新扫描目录以确认缺漏情况...");
                let refreshed_mapping = match scan_directory_bvids(&download_dir) {
                    Ok(data) => data,
                    Err(err) => {
                        println!("重新扫描下载目录失败: {err}");
                        pause_with_message("按回车返回...")?;
                        terminal::enable_raw_mode().ok();
                        return Ok(());
                    }
                };
                let refreshed_existing: Vec<String> = refreshed_mapping.keys().cloned().collect();
                let refreshed_missing = find_missing_videos(&csv_rows, &refreshed_existing);
                if refreshed_missing.is_empty() {
                    println!("{}", style("缺漏已全部补齐。").green());
                } else {
                    println!(
                        "{}",
                        style(format!("仍有 {} 个条目缺失：", refreshed_missing.len())).yellow()
                    );
                    for row in refreshed_missing {
                        if let Some(bvid) = extract_bvid(&row) {
                            println!("• {}", style(bvid).yellow());
                        }
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

fn normalize_optional(input: String) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn parse_bool_input(input: &str, default: bool) -> bool {
    match input.trim().to_lowercase().as_str() {
        "" => default,
        "y" | "yes" | "true" => true,
        "n" | "no" | "false" => false,
        _ => default,
    }
}

fn parse_args(input: &str) -> Vec<String> {
    input
        .split_whitespace()
        .map(|segment| segment.to_string())
        .collect()
}

fn resolve_string_with_default(
    input: String,
    preferred: Option<&String>,
    fallback: &str,
) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        preferred.cloned().unwrap_or_else(|| fallback.to_string())
    } else {
        trimmed.to_string()
    }
}

fn resolve_optional_with_default(input: String, preferred: Option<&String>) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        preferred.cloned()
    } else if trimmed == "-" {
        None
    } else {
        Some(trimmed.to_string())
    }
}

enum MainAction {
    NewConfig,
    UseConfig,
    EditDefaults,
    Exit,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let store = ConfigStore::load(cli.config_path.clone())?;
    let defaults = GlobalDefaultsStore::load(&store.config_dir())?;
    let mut app = App::new(store, defaults, cli.dry_run);
    app.run()
}
