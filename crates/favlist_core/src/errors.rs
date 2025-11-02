use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FavlistError {
    #[error("无法解析收藏夹链接: {0}")]
    InvalidUrl(String),
    #[error("网络请求失败: {0}")]
    Request(#[from] reqwest::Error),
    #[error("API响应错误(code={code}, message={message})")]
    Api { code: i32, message: String },
    #[error("响应不是有效的JSON: {0}")]
    InvalidJson(String),
    #[error("读取文件失败: {0}")]
    Io(#[from] io::Error),
    #[error("CSV解析失败: {0}")]
    Csv(#[from] csv::Error),
    #[error("编码转换失败: {0}")]
    Encoding(String),
    #[error("未知错误: {0}")]
    Other(String),
}

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("收藏夹导出失败: {0}")]
    Core(#[from] FavlistError),
    #[error("执行业务流程失败: {0}")]
    Context(String),
}

impl ExportError {
    pub fn context<T: Into<String>>(self, message: T) -> Self {
        let message = message.into();
        match self {
            ExportError::Core(err) => ExportError::Context(format!("{message}: {err}")),
            ExportError::Context(existing) => {
                ExportError::Context(format!("{message}: {existing}"))
            }
        }
    }
}
