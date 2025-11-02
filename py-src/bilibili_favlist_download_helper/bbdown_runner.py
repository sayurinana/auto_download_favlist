"""封装对 bbdown 的调用。"""
from __future__ import annotations

import subprocess
from pathlib import Path
from typing import Sequence


class BBDownError(RuntimeError):
    """执行 bbdown 失败。"""


def build_command(bvid: str, work_dir: Path, extra_args: Sequence[str] | None = None) -> list[str]:
    command = ["bbdown", bvid, "--work-dir", str(work_dir)]
    if extra_args:
        command.extend(extra_args)
    return command


def run_bbdown(bvid: str, work_dir: Path, *, dry_run: bool = False, extra_args: Sequence[str] | None = None) -> None:
    """执行 bbdown；在 dry-run 下仅输出命令。"""
    command = build_command(bvid, work_dir, extra_args)
    if dry_run:
        print("[dry-run]", " ".join(command))
        return
    try:
        subprocess.run(command, check=True)
    except FileNotFoundError as exc:
        raise BBDownError("未检测到 bbdown，请先安装或使用 --dry-run") from exc
    except subprocess.CalledProcessError as exc:
        raise BBDownError(f"bbdown 执行失败，退出码 {exc.returncode}") from exc
