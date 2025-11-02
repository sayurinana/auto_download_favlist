use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use dirs_next::config_dir;
use serde::{Deserialize, Serialize};

const APP_DIR: &str = "bilibili_favlist_helper";
const CONFIG_NAME: &str = "config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavConfig {
    pub fav_url: String,
    pub download_dir: String,
    pub csv_path: String,
    pub encoding: String,
    pub page_size: u32,
    pub cookie: Option<String>,
    pub timeout_secs: u64,
    pub last_synced_at: Option<String>,
    pub name: Option<String>,
}

impl FavConfig {
    pub fn download_dir_path(&self) -> PathBuf {
        PathBuf::from(&self.download_dir)
    }

    pub fn csv_path(&self) -> PathBuf {
        PathBuf::from(&self.csv_path)
    }
}

#[derive(Debug)]
pub struct ConfigStore {
    path: PathBuf,
    configs: Vec<FavConfig>,
}

impl ConfigStore {
    pub fn load(custom_path: Option<PathBuf>) -> Result<Self> {
        let path = custom_path.unwrap_or_else(default_config_path);
        let configs = if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("读取配置文件失败: {}", path.display()))?;
            serde_json::from_str(&content).with_context(|| "解析配置文件失败")?
        } else {
            Vec::new()
        };
        Ok(Self { path, configs })
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("创建配置目录失败: {}", parent.display()))?;
        }
        let json = serde_json::to_string_pretty(&self.configs)?;
        fs::write(&self.path, json).with_context(|| "写入配置文件失败")?;
        Ok(())
    }

    pub fn configs(&self) -> &[FavConfig] {
        &self.configs
    }

    pub fn add(&mut self, config: FavConfig) -> Result<()> {
        self.configs.push(config);
        self.save()
    }

    pub fn update(&mut self, index: usize, config: FavConfig) -> Result<()> {
        if index >= self.configs.len() {
            return Err(anyhow!("配置索引超出范围"));
        }
        self.configs[index] = config;
        self.save()
    }
}

fn default_config_path() -> PathBuf {
    let mut base = config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.push(APP_DIR);
    base.push(CONFIG_NAME);
    base
}
