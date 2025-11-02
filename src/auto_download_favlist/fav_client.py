"""B站收藏夹API封装。"""
from __future__ import annotations

from typing import Dict, Iterable, Iterator, Optional

import requests

from .models import FolderInfo

INFO_ENDPOINT = "https://api.bilibili.com/x/v3/fav/folder/info"
LIST_ENDPOINT = "https://api.bilibili.com/x/v3/fav/resource/list"


class BiliAPIError(RuntimeError):
    """表示B站API返回业务错误。"""

    def __init__(self, code: int, message: str, endpoint: str) -> None:
        super().__init__(f"API响应错误(code={code}, message={message}, endpoint={endpoint})")
        self.code = code
        self.message = message
        self.endpoint = endpoint


class BiliRequestError(RuntimeError):
    """表示网络请求异常。"""

    def __init__(self, reason: str, endpoint: str) -> None:
        super().__init__(f"请求{endpoint}失败: {reason}")
        self.reason = reason
        self.endpoint = endpoint


class BiliFavClient:
    """封装收藏夹相关API。"""

    def __init__(self, session: Optional[requests.Session] = None, timeout: float = 10.0) -> None:
        self.session = session or requests.Session()
        self.timeout = timeout

    def _request(self, method: str, url: str, **kwargs) -> Dict:
        try:
            response = self.session.request(method, url, timeout=self.timeout, **kwargs)
            response.raise_for_status()
        except requests.RequestException as exc:
            raise BiliRequestError(str(exc), url) from exc
        try:
            payload = response.json()
        except ValueError as exc:
            raise BiliRequestError("响应不是有效的JSON", url) from exc
        code = payload.get("code", 0)
        if code != 0:
            raise BiliAPIError(code, payload.get("message", "unknown"), url)
        return payload.get("data", {})

    def get_folder_info(self, media_id: int) -> FolderInfo:
        data = self._request("GET", INFO_ENDPOINT, params={"media_id": media_id})
        return FolderInfo(
            media_id=data.get("id", media_id),
            fid=data.get("fid", 0),
            mid=data.get("mid", 0),
            title=data.get("title", ""),
            media_count=data.get("media_count", 0),
        )

    def iter_videos(self, media_id: int, page_size: int = 40) -> Iterator[Dict]:
        """分页遍历收藏夹条目。"""
        page = 1
        while True:
            data = self._request(
                "GET",
                LIST_ENDPOINT,
                params={
                    "media_id": media_id,
                    "pn": page,
                    "ps": page_size,
                    "platform": "web",
                },
            )
            medias = data.get("medias") or []
            for item in medias:
                yield item
            if not data.get("has_more"):
                break
            page += 1

    def fetch_all_videos(self, media_id: int, page_size: int = 40) -> Iterable[Dict]:
        return list(self.iter_videos(media_id, page_size=page_size))
