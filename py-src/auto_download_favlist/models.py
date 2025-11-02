"""数据模型定义。"""
from __future__ import annotations

from dataclasses import dataclass
from typing import Optional


@dataclass(slots=True)
class FolderInfo:
    """收藏夹元信息。"""

    media_id: int
    fid: int
    mid: int
    title: str
    media_count: int


@dataclass(slots=True)
class VideoEntry:
    """收藏夹中的视频条目。"""

    bv_id: str
    title: str
    fav_title: str
    timestamp: str
    aid: Optional[int] = None
