use chrono::Local;
use url::Url;

use crate::errors::FavlistError;

const TS_FORMAT: &str = "%Y-%m-%dT%H-%M-%S";

pub fn current_timestamp() -> String {
    Local::now().format(TS_FORMAT).to_string()
}

pub fn parse_media_id(fav_url: &str) -> Result<i64, FavlistError> {
    let url = Url::parse(fav_url)
        .map_err(|err| FavlistError::InvalidUrl(format!("URL解析失败: {err}")))?;
    if let Some(query) = url.query() {
        for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
            if key == "media_id" || key == "fid" {
                let cleaned = value.trim();
                if cleaned.chars().all(|ch| ch.is_ascii_digit()) {
                    return cleaned.parse::<i64>().map_err(|_| {
                        FavlistError::InvalidUrl(format!("{key}数值无效: {cleaned}"))
                    });
                } else {
                    return Err(FavlistError::InvalidUrl(format!(
                        "{key}包含非法字符: {cleaned}"
                    )));
                }
            }
        }
    }
    Err(FavlistError::InvalidUrl(
        "未在链接查询参数中找到fid或media_id".to_string(),
    ))
}
