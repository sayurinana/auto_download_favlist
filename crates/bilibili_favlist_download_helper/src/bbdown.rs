use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

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
    child: Child,
}

impl ServeProcess {
    pub fn stop(&mut self) -> Result<()> {
        if let Err(err) = self.child.kill() {
            if err.kind() != std::io::ErrorKind::InvalidInput {
                return Err(err.into());
            }
        }
        let _ = self.child.wait();
        Ok(())
    }
}

impl Drop for ServeProcess {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

pub fn start_bbdown_serve(args: &[String]) -> Result<ServeProcess> {
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
    Ok(ServeProcess { child })
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

    pub fn remove_finished(&self) -> Result<()> {
        let url = format!("{}/remove-finished", self.base_url);
        let response = self.client.get(&url).send()?;
        if response.status() == StatusCode::OK {
            Ok(())
        } else {
            Err(anyhow!("移除已完成任务失败，状态码 {}", response.status()))
        }
    }

    pub fn wait_until_idle<F>(&self, poll: Duration, mut on_tick: F) -> Result<()>
    where
        F: FnMut(&[DownloadTask]),
    {
        loop {
            let running = self.get_running()?;
            if running.is_empty() {
                break;
            }
            on_tick(&running);
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
    #[serde(rename = "Title")]
    pub title: Option<String>,
}
