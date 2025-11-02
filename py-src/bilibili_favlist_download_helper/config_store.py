"""配置存储与数据模型。"""
from __future__ import annotations

import json
import os
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Iterable, List, Optional

APP_DIR_NAME = "bilibili_favlist_helper"
CONFIG_FILENAME = "config.json"


@dataclass(slots=True)
class FavlistConfig:
    """收藏夹配置记录。"""

    fav_url: str
    download_dir: str
    csv_path: str
    encoding: str = "utf-8"
    page_size: int = 40
    cookie: Optional[str] = None
    timeout: float = 10.0
    last_synced_at: Optional[str] = None
    name: Optional[str] = None

    @property
    def download_dir_path(self) -> Path:
        return Path(self.download_dir)

    @property
    def csv_path_path(self) -> Path:
        return Path(self.csv_path)

    def to_dict(self) -> dict:
        data = asdict(self)
        return data

    @classmethod
    def from_dict(cls, data: dict) -> "FavlistConfig":
        return cls(
            fav_url=data.get("fav_url", ""),
            download_dir=data.get("download_dir", ""),
            csv_path=data.get("csv_path", ""),
            encoding=data.get("encoding", "utf-8"),
            page_size=int(data.get("page_size", 40) or 40),
            cookie=data.get("cookie"),
            timeout=float(data.get("timeout", 10.0) or 10.0),
            last_synced_at=data.get("last_synced_at"),
            name=data.get("name"),
        )


def _resolve_config_dir() -> Path:
    base = os.environ.get("XDG_CONFIG_HOME")
    if base:
        return Path(base) / APP_DIR_NAME
    return Path.home() / ".config" / APP_DIR_NAME


def default_config_path() -> Path:
    return _resolve_config_dir() / CONFIG_FILENAME


class ConfigRepository:
    """管理收藏夹配置的读写。"""

    def __init__(self, storage_path: Optional[Path] = None) -> None:
        self.storage_path = storage_path or default_config_path()
        self._configs: List[FavlistConfig] = []
        self.load()

    def load(self) -> None:
        if not self.storage_path.exists():
            self._configs = []
            return
        try:
            content = self.storage_path.read_text(encoding="utf-8")
        except OSError:
            self._configs = []
            return
        try:
            raw = json.loads(content)
        except json.JSONDecodeError:
            self._configs = []
            return
        self._configs = [FavlistConfig.from_dict(item) for item in raw or []]

    def save(self) -> None:
        self.storage_path.parent.mkdir(parents=True, exist_ok=True)
        data = [cfg.to_dict() for cfg in self._configs]
        self.storage_path.write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8")

    def list(self) -> List[FavlistConfig]:
        return list(self._configs)

    def add(self, config: FavlistConfig) -> None:
        self._configs.append(config)
        self.save()

    def update(self, index: int, config: FavlistConfig) -> None:
        self._configs[index] = config
        self.save()

    def remove(self, index: int) -> FavlistConfig:
        cfg = self._configs.pop(index)
        self.save()
        return cfg

    def replace_all(self, configs: Iterable[FavlistConfig]) -> None:
        self._configs = list(configs)
        self.save()

    def get(self, index: int) -> FavlistConfig:
        return self._configs[index]

    def is_empty(self) -> bool:
        return not self._configs
