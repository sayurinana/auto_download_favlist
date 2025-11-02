"""交互式收藏夹下载助手 CLI。"""
from __future__ import annotations

from pathlib import Path
from typing import Optional

import typer

from auto_download_favlist.utils import current_timestamp

from .bbdown_runner import BBDownError, run_bbdown
from .bv_tools import diff_new_entries, find_missing_videos, read_csv_rows, scan_directory_bvids, write_inventory_file
from .config_store import ConfigRepository, FavlistConfig
from .export_service import ExportError, ExportResult, export_to_csv

app = typer.Typer(add_completion=False, help="B 站收藏夹下载辅助工具")


def _resolve_download_dir(input_value: str) -> Path:
    directory = Path(input_value).expanduser().resolve()
    directory.mkdir(parents=True, exist_ok=True)
    return directory


def _prompt_non_empty(message: str) -> str:
    while True:
        value = typer.prompt(message).strip()
        if value:
            return value
        typer.secho("输入不能为空，请重试。", fg=typer.colors.RED)


def _select_config(repo: ConfigRepository) -> Optional[int]:
    configs = repo.list()
    if not configs:
        typer.secho("当前没有已保存的收藏夹配置，请先录入。", fg=typer.colors.YELLOW)
        return None
    typer.echo("已保存的配置：")
    for idx, cfg in enumerate(configs, start=1):
        title = cfg.name or f"收藏夹 {idx}"
        typer.echo(f"{idx}. {title} -> {cfg.fav_url}")
    selection = typer.prompt("请输入要操作的序号", default="").strip()
    if not selection:
        typer.secho("未选择任一配置。", fg=typer.colors.YELLOW)
        return None
    if not selection.isdigit():
        typer.secho("请输入有效的数字序号。", fg=typer.colors.RED)
        return None
    index = int(selection) - 1
    if index < 0 or index >= len(configs):
        typer.secho("序号超出范围。", fg=typer.colors.RED)
        return None
    return index


def _export_for_config(
    config: FavlistConfig,
    download_dir: Path,
    *,
    target_csv: Optional[Path] = None,
    custom_timestamp: Optional[str] = None,
) -> ExportResult:
    timestamp = custom_timestamp or current_timestamp()
    csv_path = target_csv or download_dir / f"{timestamp}-favlist.csv"
    return export_to_csv(
        config.fav_url,
        csv_path,
        encoding=config.encoding,
        page_size=config.page_size,
        cookie=config.cookie,
        timeout=config.timeout,
        timestamp=timestamp,
    )


def _handle_bbdown(bvids: list[str], work_dir: Path, *, dry_run: bool) -> None:
    for bvid in bvids:
        try:
            run_bbdown(bvid, work_dir, dry_run=dry_run)
        except BBDownError as exc:
            typer.secho(str(exc), fg=typer.colors.RED)


def _flow_new_config(repo: ConfigRepository, *, dry_run: bool) -> None:
    typer.echo("录入新收藏夹：")
    fav_url = _prompt_non_empty("请输入收藏夹 URL")
    default_dir = str(Path.cwd())
    download_dir_input = typer.prompt("请输入下载目录路径", default=default_dir).strip()
    if not download_dir_input:
        download_dir_input = default_dir
    download_dir = _resolve_download_dir(download_dir_input)
    timestamp = current_timestamp()
    csv_path = download_dir / f"{timestamp}-favlist.csv"
    typer.echo(f"CSV 文件将保存至：{csv_path}")

    config = FavlistConfig(
        fav_url=fav_url,
        download_dir=str(download_dir),
        csv_path=str(csv_path),
        last_synced_at=timestamp,
    )
    name_input = typer.prompt("可选的配置名称(可留空)", default="").strip()
    if name_input:
        config.name = name_input

    try:
        result = _export_for_config(config, download_dir, target_csv=csv_path, custom_timestamp=timestamp)
    except ExportError as exc:
        typer.secho(f"抓取收藏夹失败：{exc}", fg=typer.colors.RED)
        return

    repo.add(
        FavlistConfig(
            fav_url=config.fav_url,
            download_dir=str(download_dir),
            csv_path=str(result.csv_path),
            encoding=config.encoding,
            page_size=config.page_size,
            cookie=config.cookie,
            timeout=config.timeout,
            last_synced_at=result.timestamp,
            name=config.name,
        )
    )
    typer.secho("录入并抓取完成。", fg=typer.colors.GREEN)


def _flow_edit_config(repo: ConfigRepository, index: int) -> None:
    config = repo.get(index)
    typer.echo("编辑配置，直接回车表示保持现值。")
    typer.echo(f"当前下载目录：{config.download_dir}")
    download_dir_input = typer.prompt("新的下载目录", default="").strip()
    if download_dir_input:
        download_dir = _resolve_download_dir(download_dir_input)
        config.download_dir = str(download_dir)
    typer.echo(f"当前收藏夹 URL：{config.fav_url}")
    fav_url_input = typer.prompt("新的收藏夹 URL", default="").strip()
    if fav_url_input:
        config.fav_url = fav_url_input
    typer.echo(f"当前 CSV 路径：{config.csv_path}")
    csv_path_input = typer.prompt("新的 CSV 路径 (绝对路径)", default="").strip()
    if csv_path_input:
        config.csv_path = str(Path(csv_path_input).expanduser().resolve())
    typer.echo(f"当前显示名称：{config.name or '(未设置)'}")
    name_input = typer.prompt("新的展示名称 (输入-清除)", default="").strip()
    if name_input == "-":
        config.name = None
    elif name_input:
        config.name = name_input
    typer.echo(f"当前编码：{config.encoding}")
    encoding_input = typer.prompt("新的编码", default="").strip()
    if encoding_input:
        config.encoding = encoding_input
    typer.echo(f"当前 page_size：{config.page_size}")
    page_size_input = typer.prompt("新的 page_size", default="").strip()
    if page_size_input.isdigit():
        config.page_size = int(page_size_input)
    typer.echo(f"当前超时时间：{config.timeout}")
    timeout_input = typer.prompt("新的超时时间 (秒)", default="").strip()
    if timeout_input:
        try:
            config.timeout = float(timeout_input)
        except ValueError:
            typer.secho("超时时间格式无效，保持原值。", fg=typer.colors.YELLOW)
    typer.echo(f"当前 Cookie：{config.cookie or '(未设置)'}")
    cookie_input = typer.prompt("新的 Cookie (输入-清除)", default="").strip()
    if cookie_input == "-":
        config.cookie = None
    elif cookie_input:
        config.cookie = cookie_input
    repo.update(index, config)
    typer.secho("配置已更新。", fg=typer.colors.GREEN)


def _flow_check_update(repo: ConfigRepository, index: int, *, dry_run: bool) -> None:
    config = repo.get(index)
    old_csv_path = Path(config.csv_path)
    backup_path = old_csv_path.with_suffix(".backup.csv")
    old_rows = read_csv_rows(old_csv_path, config.encoding)
    backup_taken = False
    if old_csv_path.exists():
        backup_path.parent.mkdir(parents=True, exist_ok=True)
        if backup_path.exists():
            backup_path.unlink()
        old_csv_path.rename(backup_path)
        backup_taken = True
        typer.echo(f"旧 CSV 已备份到：{backup_path}")
    else:
        typer.echo("未找到旧 CSV，将直接抓取最新数据。")

    download_dir = Path(config.download_dir).expanduser().resolve()
    download_dir.mkdir(parents=True, exist_ok=True)
    timestamp = current_timestamp()
    new_csv_path = download_dir / f"{timestamp}-favlist.csv"

    try:
        result = _export_for_config(config, download_dir, target_csv=new_csv_path, custom_timestamp=timestamp)
    except ExportError as exc:
        typer.secho(f"抓取失败：{exc}", fg=typer.colors.RED)
        if backup_taken and backup_path.exists():
            backup_path.rename(old_csv_path)
            typer.echo("已恢复原 CSV。")
        return

    new_rows = read_csv_rows(result.csv_path, config.encoding)
    diffs = diff_new_entries(old_rows, new_rows)
    if diffs:
        typer.secho(f"发现 {len(diffs)} 个新增条目，将依次下载。", fg=typer.colors.CYAN)
        missing_bvids = [row.get("bv_id") or row.get("BV号") or row.get("视频BV号") for row in diffs]
        missing_bvids = [b for b in missing_bvids if b]
        _handle_bbdown(missing_bvids, download_dir, dry_run=dry_run)
    else:
        typer.secho("未检测到新增条目。", fg=typer.colors.GREEN)

    config.csv_path = str(result.csv_path)
    config.last_synced_at = result.timestamp
    repo.update(index, config)
    typer.secho("更新流程完成。", fg=typer.colors.GREEN)


def _flow_check_missing(repo: ConfigRepository, index: int, *, dry_run: bool) -> None:
    config = repo.get(index)
    download_dir = Path(config.download_dir).expanduser().resolve()
    if not download_dir.exists():
        typer.secho("下载目录不存在，请先更新路径或重新录入。", fg=typer.colors.RED)
        return
    mapping = scan_directory_bvids(download_dir)
    inventory_path = write_inventory_file(download_dir, mapping)
    typer.echo(f"已生成目录清单：{inventory_path}")

    csv_rows = read_csv_rows(Path(config.csv_path), config.encoding)
    existing_bvids = sorted(mapping.keys())
    missing = find_missing_videos(csv_rows, existing_bvids)
    if not missing:
        typer.secho("未检测到缺漏视频。", fg=typer.colors.GREEN)
        return

    typer.secho(f"检测到 {len(missing)} 个缺失条目，开始下载。", fg=typer.colors.CYAN)
    missing_bvids = [row.get("bv_id") or row.get("BV号") or row.get("视频BV号") for row in missing]
    missing_bvids = [b for b in missing_bvids if b]
    _handle_bbdown(missing_bvids, download_dir, dry_run=dry_run)


def _display_main_menu() -> str:
    typer.echo("\n请选择操作：")
    typer.echo("1. 录入新收藏夹")
    typer.echo("2. 使用存档配置")
    typer.echo("0. 退出程序")
    choice = typer.prompt("输入选项", default="").strip()
    return choice


def _display_saved_menu() -> str:
    typer.echo("\n存档操作：")
    typer.echo("1. 编辑配置")
    typer.echo("2. 检查更新")
    typer.echo("3. 检查缺漏")
    typer.echo("0. 返回上层")
    return typer.prompt("输入选项", default="").strip()


@app.command()
def run(
    config_path: Optional[Path] = typer.Option(None, "--config-path", help="自定义配置文件路径"),
    dry_run: bool = typer.Option(False, "--dry-run", help="仅打印将要执行的命令，不实际下载"),
) -> None:
    """启动交互式助手。"""
    repo = ConfigRepository(config_path)
    typer.echo("Bilibili 收藏夹下载助手启动。输入数字选择操作。")
    if dry_run:
        typer.secho("当前处于 dry-run 模式，不会执行实际下载。", fg=typer.colors.YELLOW)

    while True:
        choice = _display_main_menu()
        if choice == "1":
            _flow_new_config(repo, dry_run=dry_run)
        elif choice == "2":
            index = _select_config(repo)
            if index is None:
                continue
            while True:
                sub_choice = _display_saved_menu()
                if sub_choice == "1":
                    _flow_edit_config(repo, index)
                elif sub_choice == "2":
                    _flow_check_update(repo, index, dry_run=dry_run)
                elif sub_choice == "3":
                    _flow_check_missing(repo, index, dry_run=dry_run)
                elif sub_choice == "0":
                    break
                else:
                    typer.secho("未知选项，请重新选择。", fg=typer.colors.YELLOW)
        elif choice == "0":
            typer.echo("已退出助手。")
            raise typer.Exit(code=0)
        else:
            typer.secho("无效输入，请重新选择。", fg=typer.colors.RED)


def main() -> None:
    app()
