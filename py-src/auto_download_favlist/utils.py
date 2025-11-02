"""通用工具方法。"""
from __future__ import annotations

from datetime import datetime, timezone
from typing import Dict, Optional
from urllib.parse import parse_qs, urlparse

import requests

TS_FORMAT = "%Y-%m-%dT%H-%M-%S"
_DEFAULT_HEADERS = {
    "User-Agent": (
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
        "AppleWebKit/537.36 (KHTML, like Gecko) "
        "Chrome/118.0 Safari/537.36"
    ),
    "Referer": "https://www.bilibili.com/",
}


def extract_media_id(url: str) -> int:
    """从收藏夹URL中提取`media_id`/`fid`。"""
    parsed = urlparse(url)
    query = parse_qs(parsed.query)
    candidates = query.get("fid") or query.get("media_id")
    if not candidates:
        raise ValueError("无法在链接查询参数中找到fid或media_id")
    value = candidates[0].strip()
    if not value.isdigit():
        raise ValueError(f"解析到的fid值无效: {value}")
    return int(value)


def build_session(cookie: Optional[str] = None, extra_headers: Optional[Dict[str, str]] = None) -> requests.Session:
    """构造带默认Headers的requests会话。"""
    session = requests.Session()
    session.headers.update(_DEFAULT_HEADERS)
    if cookie:
        session.headers["Cookie"] = cookie
    if extra_headers:
        session.headers.update(extra_headers)
    return session


def current_timestamp() -> str:
    """返回当前时间戳，格式为`YYYY-MM-ddTHH-mm-ss`。"""
    return datetime.now(timezone.utc).astimezone().strftime(TS_FORMAT)
