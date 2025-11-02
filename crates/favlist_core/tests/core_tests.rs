use std::fs;

use favlist_core::{
    export_favlist, load_existing_bv_ids, parse_media_id, read_csv_rows, write_entries,
    ExportOptions, VideoEntry,
};
use httpmock::prelude::*;
use serde_json::json;
use tempfile::tempdir;

type TestResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn parse_media_id_success() {
    let url = "https://space.bilibili.com/234561771/favlist?fid=3670113371";
    let media_id = parse_media_id(url).expect("media id");
    assert_eq!(media_id, 3670113371);
}

#[test]
fn parse_media_id_invalid() {
    let url = "https://space.bilibili.com/234561771/favlist";
    let err = parse_media_id(url).unwrap_err();
    assert!(err.to_string().contains("fid"));
}

#[test]
fn write_and_read_csv_with_gbk_encoding() -> TestResult<()> {
    let dir = tempdir()?;
    let csv_path = dir.path().join("favlist.csv");
    let entries = vec![VideoEntry {
        bv_id: "BV1xx41117xb".to_string(),
        title: "测试标题".to_string(),
        fav_title: "收藏夹".to_string(),
        timestamp: "2025-11-02T12-00-00".to_string(),
        aid: Some(12345),
    }];

    write_entries(&csv_path, "gbk", &entries)?;

    let existing = load_existing_bv_ids(&csv_path, "gbk")?;
    assert!(existing.contains("BV1xx41117xb"));

    let rows = read_csv_rows(&csv_path, "gbk")?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get("title").unwrap(), "测试标题");

    Ok(())
}

#[tokio::test]
async fn export_favlist_writes_new_entries() -> TestResult<()> {
    let server = MockServer::start();

    let info_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/x/v3/fav/folder/info")
            .query_param("media_id", "3670113371");
        then.status(200).json_body(json!({
            "code": 0,
            "data": {
                "id": 3670113371_i64,
                "fid": 111,
                "mid": 222,
                "title": "示例收藏夹",
                "media_count": 2
            }
        }));
    });

    let list_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/x/v3/fav/resource/list")
            .query_param("media_id", "3670113371")
            .query_param("pn", "1")
            .query_param("ps", "40")
            .query_param("platform", "web");
        then.status(200).json_body(json!({
            "code": 0,
            "data": {
                "medias": [
                    {"bvid": "BV1xx41117xb", "title": "视频一", "id": 987654_i64},
                    {"bvid": "BV1yy41117xy", "title": "视频二", "id": 987655_i64}
                ],
                "has_more": false
            }
        }));
    });

    let dir = tempdir()?;
    let csv_path = dir.path().join("favlist.csv");

    let options = ExportOptions {
        fav_url: "https://space.bilibili.com/234561771/favlist?fid=3670113371".to_string(),
        csv_path: csv_path.clone(),
        encoding: "utf-8".to_string(),
        page_size: 40,
        cookie: None,
        timeout_secs: 10,
        timestamp: Some("2025-11-02T12-00-00".to_string()),
        extra_headers: Default::default(),
        base_url: Some(server.base_url()),
    };

    let result = export_favlist(options.clone()).await?;
    assert_eq!(result.new_entries.len(), 2);
    assert_eq!(result.folder_info.title, "示例收藏夹");

    let content = fs::read_to_string(&csv_path)?;
    assert!(content.contains("BV1xx41117xb"));
    assert!(content.contains("视频一"));

    info_mock.assert();
    list_mock.assert();

    // Re-run export to ensure duplicates are skipped
    let second = export_favlist(options).await?;
    assert!(second.new_entries.is_empty());

    Ok(())
}
