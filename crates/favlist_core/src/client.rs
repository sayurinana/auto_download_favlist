use std::collections::HashMap;
use std::time::Duration;

use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client,
};
use serde::de::DeserializeOwned;

use crate::errors::FavlistError;
use crate::models::{ApiResponse, FolderInfo, FolderInfoPayload, ResourceListPayload};

pub const INFO_ENDPOINT: &str = "https://api.bilibili.com/x/v3/fav/folder/info";
pub const LIST_ENDPOINT: &str = "https://api.bilibili.com/x/v3/fav/resource/list";

pub const DEFAULT_HEADERS: [(&str, &str); 2] = [
    (
        "user-agent",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \\nAppleWebKit/537.36 (KHTML, like Gecko) \\nChrome/118.0 Safari/537.36",
    ),
    ("referer", "https://www.bilibili.com/"),
];

#[derive(Debug, Clone)]
pub struct ClientOptions {
    pub timeout: Duration,
    pub cookie: Option<String>,
    pub extra_headers: HashMap<String, String>,
}

impl Default for ClientOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            cookie: None,
            extra_headers: HashMap::new(),
        }
    }
}

#[derive(Clone)]
pub struct BiliFavClient {
    client: Client,
    options: ClientOptions,
}

impl BiliFavClient {
    pub fn new(options: ClientOptions) -> Result<Self, FavlistError> {
        let mut headers = HeaderMap::new();
        for (name, value) in DEFAULT_HEADERS.iter() {
            headers.insert(
                HeaderName::from_static(name),
                HeaderValue::from_static(value),
            );
        }
        if let Some(cookie) = &options.cookie {
            headers.insert(
                HeaderName::from_static("cookie"),
                HeaderValue::from_str(cookie)
                    .map_err(|err| FavlistError::InvalidUrl(format!("Cookie格式无效: {err}")))?,
            );
        }
        for (key, value) in &options.extra_headers {
            let header_name = HeaderName::from_bytes(key.as_bytes())
                .map_err(|err| FavlistError::InvalidUrl(format!("Header名无效: {err}")))?;
            let header_value = HeaderValue::from_str(value)
                .map_err(|err| FavlistError::InvalidUrl(format!("Header值无效: {err}")))?;
            headers.insert(header_name, header_value);
        }

        let client = Client::builder()
            .timeout(options.timeout)
            .default_headers(headers)
            .build()
            .map_err(FavlistError::Request)?;

        Ok(Self { client, options })
    }

    pub async fn get_folder_info(&self, media_id: i64) -> Result<FolderInfo, FavlistError> {
        let payload: FolderInfoPayload = self
            .request(INFO_ENDPOINT, &[("media_id", media_id.to_string())])
            .await?;
        Ok(payload.into_folder_info(media_id))
    }

    pub async fn list_videos(
        &self,
        media_id: i64,
        page_size: u32,
    ) -> Result<Vec<ResourceListPayload>, FavlistError> {
        let mut page = 1u32;
        let mut pages = Vec::new();
        loop {
            let payload: ResourceListPayload = self
                .request(
                    LIST_ENDPOINT,
                    &[
                        ("media_id", media_id.to_string()),
                        ("pn", page.to_string()),
                        ("ps", page_size.to_string()),
                        ("platform", "web".to_string()),
                    ],
                )
                .await?;
            let has_more = payload.has_more;
            pages.push(payload);
            if !has_more {
                break;
            }
            page += 1;
        }
        Ok(pages)
    }

    async fn request<T: DeserializeOwned>(
        &self,
        url: &str,
        params: &[(&str, String)],
    ) -> Result<T, FavlistError> {
        let mut req = self.client.get(url);
        for (k, v) in params {
            req = req.query(&[(k, v.as_str())]);
        }
        let response = req.send().await.map_err(FavlistError::Request)?;
        let status = response.status();
        if !status.is_success() {
            return Err(FavlistError::Other(format!("HTTP请求失败: {status}")));
        }
        let bytes = response.bytes().await.map_err(FavlistError::Request)?;
        let payload: ApiResponse<T> = serde_json::from_slice(&bytes)
            .map_err(|err| FavlistError::InvalidJson(err.to_string()))?;
        if payload.code != 0 {
            return Err(FavlistError::Api {
                code: payload.code,
                message: payload.message.unwrap_or_else(|| "unknown".to_string()),
            });
        }
        payload
            .data
            .ok_or_else(|| FavlistError::InvalidJson("响应缺少data字段".to_string()))
    }

    pub fn options(&self) -> &ClientOptions {
        &self.options
    }
}
