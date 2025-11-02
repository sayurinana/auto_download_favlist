"""CSV读写与去重逻辑。"""
from __future__ import annotations

import csv
from pathlib import Path
from typing import Iterable, Sequence, Set

from .models import VideoEntry

FIELDNAMES = ["timestamp", "bv_id", "title", "fav_name"]


def load_existing_bv_ids(path: Path, encoding: str) -> Set[str]:
    """读取已存在的BV号集合。"""
    if not path.exists():
        return set()
    existing: Set[str] = set()
    try:
        with path.open("r", encoding=encoding, newline="") as fh:
            reader = csv.DictReader(fh)
            candidate_fields = {"bv_id", "BV号", "视频BV号"}
            bv_field = next((f for f in reader.fieldnames or [] if f in candidate_fields), "bv_id")
            for row in reader:
                value = (row.get(bv_field) or "").strip()
                if value:
                    existing.add(value)
    except UnicodeDecodeError as exc:
        raise ValueError("读取CSV失败，请检查--encoding参数是否正确") from exc
    return existing


def write_entries(path: Path, encoding: str, entries: Sequence[VideoEntry]) -> int:
    """将新条目追加写入CSV。"""
    if not entries:
        return 0
    path.parent.mkdir(parents=True, exist_ok=True)
    is_new_file = not path.exists()
    with path.open("a", encoding=encoding, newline="") as fh:
        writer = csv.DictWriter(fh, fieldnames=FIELDNAMES)
        if is_new_file:
            writer.writeheader()
        for entry in entries:
            writer.writerow(
                {
                    "timestamp": entry.timestamp,
                    "bv_id": entry.bv_id,
                    "title": entry.title,
                    "fav_name": entry.fav_title,
                }
            )
    return len(entries)
