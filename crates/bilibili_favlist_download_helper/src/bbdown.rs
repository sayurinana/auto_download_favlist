use std::collections::HashSet;
use std::env;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};
use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

pub fn run_bbdown(bvid: &str, work_dir: &Path, dry_run: bool) -> Result<()> {
    if dry_run {
        println!(
            "[dry-run] bbdown {} --work-dir {}",
            bvid,
            work_dir.display()
        );
        return Ok(());
    }

    let status = Command::new("bbdown")
        .arg(bvid)
        .arg("--work-dir")
        .arg(work_dir)
        .status()
        .with_context(|| "执行 bbdown 失败")?;

    if !status.success() {
        let code = status.code().unwrap_or(-1);
        bail!("bbdown 执行失败，退出码 {code}");
    }

    Ok(())
}

pub struct ServeProcess {
    child: Option<Child>,
    windows_pid: Option<u32>,
}

impl ServeProcess {
    pub fn stop(&mut self) -> Result<()> {
        if let Some(child) = self.child.as_mut() {
            if let Err(err) = child.kill() {
                if err.kind() != std::io::ErrorKind::InvalidInput {
                    return Err(err.into());
                }
            }
            let _ = child.wait();
            self.child = None;
        }

        if let Some(pid) = self.windows_pid.take() {
            let status = Command::new("taskkill.exe")
                .args(["/PID", &pid.to_string(), "/F"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .with_context(|| "关闭 bbdown serve 失败")?;
            if !status.success() {
                eprintln!(
                    "taskkill.exe 退出码 {:?}, 可能需要手动关闭窗口",
                    status.code()
                );
            }
        }

        Ok(())
    }
}

impl Drop for ServeProcess {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

pub fn start_bbdown_serve(args: &[String]) -> Result<ServeProcess> {
    if should_launch_windows_detached() {
        start_windows_bbdown_serve(args)
    } else {
        start_native_bbdown_serve(args)
    }
}

fn start_native_bbdown_serve(args: &[String]) -> Result<ServeProcess> {
    let mut command = Command::new("bbdown");
    command.arg("serve");
    for arg in args {
        command.arg(arg);
    }
    let child = command
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| "启动 bbdown serve 失败")?;
    Ok(ServeProcess {
        child: Some(child),
        windows_pid: None,
    })
}

fn start_windows_bbdown_serve(args: &[String]) -> Result<ServeProcess> {
    let mut full_args = Vec::with_capacity(args.len() + 1);
    full_args.push("serve".to_string());
    full_args.extend(args.iter().cloned());
    let escaped = full_args
        .iter()
        .map(|arg| arg.replace('\'', "''"))
        .collect::<Vec<_>>()
        .join(" ");
    let ps_command = format!(
        "$proc = Start-Process -FilePath 'bbdown' -ArgumentList '{}' -WindowStyle Normal -PassThru; Write-Output $proc.Id",
        escaped
    );
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-Command", &ps_command])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .with_context(|| "启动 bbdown serve 失败")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("启动 bbdown serve 失败: {}", stderr.trim());
    }

    let pid_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let pid: u32 = pid_str
        .parse()
        .with_context(|| format!("解析 bbdown serve PID 失败: {pid_str}"))?;

    Ok(ServeProcess {
        child: None,
        windows_pid: Some(pid),
    })
}

fn should_launch_windows_detached() -> bool {
    cfg!(target_os = "windows") || env::var("WSL_DISTRO_NAME").is_ok()
}

#[derive(Clone)]
pub struct BbdownApiClient {
    client: Client,
    base_url: String,
}

impl BbdownApiClient {
    pub fn new(base_url: &str, timeout: Duration) -> Result<Self> {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .with_context(|| "初始化 BBDown API 客户端失败")?;
        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }

    pub fn add_task(
        &self,
        url_or_bvid: &str,
        file_pattern: Option<&str>,
        multi_file_pattern: Option<&str>,
    ) -> Result<()> {
        let url = format!("{}/add-task", self.base_url);
        let payload = AddTaskPayload {
            url: url_or_bvid,
            file_pattern,
            multi_file_pattern,
        };
        let response = self.client.post(&url).json(&payload).send()?;
        if response.status() == StatusCode::OK {
            Ok(())
        } else {
            Err(anyhow!("添加任务失败，状态码 {}", response.status()))
        }
    }

    pub fn get_running(&self) -> Result<Vec<DownloadTask>> {
        let url = format!("{}/get-tasks/running", self.base_url);
        let response = self.client.get(&url).send()?;
        if response.status() == StatusCode::OK {
            Ok(response.json().with_context(|| "解析运行中任务失败")?)
        } else {
            Err(anyhow!("获取运行中任务失败，状态码 {}", response.status()))
        }
    }

    pub fn get_finished(&self) -> Result<Vec<DownloadTask>> {
        let url = format!("{}/get-tasks/finished", self.base_url);
        let response = self.client.get(&url).send()?;
        if response.status() == StatusCode::OK {
            Ok(response.json().with_context(|| "解析已完成任务失败")?)
        } else {
            Err(anyhow!("获取已完成任务失败，状态码 {}", response.status()))
        }
    }

    pub fn remove_finished(&self) -> Result<()> {
        let url = format!("{}/remove-finished", self.base_url);
        let response = self.client.get(&url).send()?;
        if response.status() == StatusCode::OK {
            Ok(())
        } else {
            Err(anyhow!("移除已完成任务失败，状态码 {}", response.status()))
        }
    }

    pub fn wait_until_idle<F>(
        &self,
        poll: Duration,
        targets: &[String],
        mut on_tick: F,
    ) -> Result<()>
    where
        F: FnMut(&[DownloadTask]),
    {
        let mut pending: HashSet<String> = targets.iter().map(|t| normalize_target(t)).collect();
        let mut stable_empty = 0usize;
        let mut last_change = Instant::now();
        let timeout = Duration::from_secs(600);
        loop {
            let running = self.get_running()?;
            let mut target_still_running = false;
            for task in &running {
                if let Some(key) = task.target_key() {
                    if pending.contains(&key) {
                        target_still_running = true;
                        break;
                    }
                }
            }
            on_tick(&running);

            if !pending.is_empty() {
                let finished = self.get_finished()?;
                for task in finished {
                    if let Some(key) = task.target_key() {
                        if pending.remove(&key) {
                            last_change = Instant::now();
                        }
                    }
                }
            }

            if pending.is_empty() && running.is_empty() {
                stable_empty += 1;
                if stable_empty >= 2 {
                    break;
                }
            } else {
                stable_empty = 0;
            }

            if last_change.elapsed() > timeout {
                return Err(anyhow!("等待下载任务超时, 请确认 bbdown serve 状态"));
            }

            if !target_still_running && pending.is_empty() && running.is_empty() {
                break;
            }

            thread::sleep(poll);
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct AddTaskPayload<'a> {
    #[serde(rename = "Url")]
    pub url: &'a str,
    #[serde(rename = "FilePattern", skip_serializing_if = "Option::is_none")]
    pub file_pattern: Option<&'a str>,
    #[serde(rename = "MultiFilePattern", skip_serializing_if = "Option::is_none")]
    pub multi_file_pattern: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
pub struct DownloadTask {
    #[serde(rename = "Aid")]
    pub aid: Option<String>,
    #[serde(rename = "Url")]
    pub url: Option<String>,
    #[serde(rename = "Title")]
    pub title: Option<String>,
}

impl DownloadTask {
    fn target_key(&self) -> Option<String> {
        self.url
            .as_ref()
            .map(|value| normalize_target(value))
            .or_else(|| self.aid.as_ref().map(|value| normalize_target(value)))
    }
}

fn normalize_target(value: &str) -> String {
    value.trim().to_lowercase()
}
