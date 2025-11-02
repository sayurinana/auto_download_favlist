"""封装收藏夹导出逻辑。"""
from __future__ import annotations

import csv
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

from auto_download_favlist.csv_writer import FIELDNAMES, load_existing_bv_ids, write_entries
from auto_download_favlist.fav_client import BiliAPIError, BiliFavClient, BiliRequestError
from auto_download_favlist.models import FolderInfo, VideoEntry
from auto_download_favlist.utils import build_session, current_timestamp, extract_media_id


class ExportError(RuntimeError):
    """导出收藏夹时的错误。"""


@dataclass(slots=True)
class ExportResult:
    """导出结果。"""

    csv_path: Path
    folder_info: FolderInfo
    new_entries: list[VideoEntry]
    timestamp: str


def export_to_csv(
    fav_url: str,
    csv_path: Path,
    *,
    encoding: str = "utf-8",
    page_size: int = 40,
    cookie: Optional[str] = None,
    timeout: float = 10.0,
    timestamp: Optional[str] = None,
) -> ExportResult:
    """导出指定收藏夹到 CSV。"""
    try:
        media_id = extract_media_id(fav_url)
    except ValueError as exc:
        raise ExportError(f"无法解析收藏夹链接: {exc}") from exc

    session = build_session(cookie=cookie)
    client = BiliFavClient(session=session, timeout=timeout)

    try:
        folder_info = client.get_folder_info(media_id)
    except (BiliAPIError, BiliRequestError) as exc:
        raise ExportError(f"获取收藏夹信息失败: {exc}") from exc

    try:
        medias = list(client.iter_videos(media_id, page_size=page_size))
    except (BiliAPIError, BiliRequestError) as exc:
        raise ExportError(f"抓取收藏夹条目失败: {exc}") from exc

    existing_ids = load_existing_bv_ids(csv_path, encoding)
    ts = timestamp or current_timestamp()
    new_entries: list[VideoEntry] = []
    for item in medias:
        bv_id = (item.get("bv_id") or item.get("bvid") or "").strip()
        if not bv_id:
            aid = item.get("id")
            if aid:
                bv_id = f"av{aid}"
            else:
                continue
        if bv_id in existing_ids:
            continue
        existing_ids.add(bv_id)
        title = item.get("title", "").strip()
        new_entries.append(
            VideoEntry(
                bv_id=bv_id,
                title=title,
                fav_title=folder_info.title,
                timestamp=ts,
                aid=item.get("id"),
            )
        )

    if new_entries:
        write_entries(csv_path, encoding, new_entries)
    else:
        # 如果没有新条目且文件不存在，仍需创建带表头的 CSV
        if not csv_path.exists():
            csv_path.parent.mkdir(parents=True, exist_ok=True)
            with csv_path.open(\"w\", encoding=encoding, newline=\"\") as fh:
                writer = csv.DictWriter(fh, fieldnames=FIELDNAMES)
                writer.writeheader()

    return ExportResult(csv_path=csv_path, folder_info=folder_info, new_entries=new_entries, timestamp=ts)
