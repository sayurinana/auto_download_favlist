"""BV号解析与CSV对比工具。"""
from __future__ import annotations

import csv
import re
from pathlib import Path
from typing import Dict, Iterable, List, Sequence, Set, Tuple

BV_PATTERN = re.compile(r"(BV[0-9A-Za-z]{10})")


def extract_bvids(text: str) -> Set[str]:
    """从文本中提取所有 BV 号。"""
    return {match.group(1) for match in BV_PATTERN.finditer(text)}


def scan_directory_bvids(directory: Path) -> Dict[str, List[Path]]:
    """扫描目录下所有文件名，构造 BV 号到文件路径的映射。"""
    mapping: Dict[str, List[Path]] = {}
    for file_path in directory.rglob("*"):
        if not file_path.is_file():
            continue
        ids = extract_bvids(file_path.name)
        for bvid in ids:
            mapping.setdefault(bvid, []).append(file_path)
    return mapping


def write_inventory_file(directory: Path, mapping: Dict[str, List[Path]]) -> Path:
    """将扫描结果写入文本文件，返回文件路径。"""
    inventory_path = directory / "existing_videos.txt"
    lines: List[str] = ["# 文件名及对应BV号列表"]
    for bvid, paths in sorted(mapping.items()):
        for p in paths:
            lines.append(f"{bvid}	{p.name}")
    if len(lines) == 1:
        lines.append("(未找到包含BV号的文件)")
    inventory_path.write_text("\n".join(lines), encoding="utf-8")
    return inventory_path


def read_csv_rows(csv_path: Path, encoding: str = "utf-8") -> List[Dict[str, str]]:
    """读取CSV内容为字典列表。"""
    rows: List[Dict[str, str]] = []
    if not csv_path.exists():
        return rows
    with csv_path.open("r", encoding=encoding, newline="") as fh:
        reader = csv.DictReader(fh)
        for row in reader:
            rows.append({k or "": (v or "").strip() for k, v in row.items()})
    return rows


def extract_bvids_from_rows(rows: Sequence[Dict[str, str]]) -> Set[str]:
    """从CSV行中提取 BV 集合。"""
    bvids: Set[str] = set()
    candidate_fields = ("bv_id", "BV号", "视频BV号")
    for row in rows:
        for field in candidate_fields:
            value = row.get(field)
            if value:
                bvids.add(value.strip())
                break
    return bvids


def diff_new_entries(
    old_rows: Sequence[Dict[str, str]],
    new_rows: Sequence[Dict[str, str]],
) -> List[Dict[str, str]]:
    """找出新CSV中相对于旧CSV新增的条目。"""
    old_bvids = extract_bvids_from_rows(old_rows)
    diffs: List[Dict[str, str]] = []
    for row in new_rows:
        bvid = row.get("bv_id") or row.get("BV号") or row.get("视频BV号")
        if not bvid:
            continue
        if bvid not in old_bvids:
            diffs.append(row)
    return diffs


def find_missing_videos(
    csv_rows: Sequence[Dict[str, str]],
    existing_bvids: Iterable[str],
) -> List[Dict[str, str]]:
    """找出CSV中缺少本地文件的条目。"""
    existing_set = set(existing_bvids)
    missing: List[Dict[str, str]] = []
    for row in csv_rows:
        bvid = row.get("bv_id") or row.get("BV号") or row.get("视频BV号")
        if not bvid:
            continue
        if bvid not in existing_set:
            missing.append(row)
    return missing
