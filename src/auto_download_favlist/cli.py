"""命令行入口。"""
from __future__ import annotations

from pathlib import Path
from typing import Optional

import typer

from .csv_writer import load_existing_bv_ids, write_entries
from .fav_client import BiliAPIError, BiliFavClient, BiliRequestError
from .models import VideoEntry
from .utils import build_session, current_timestamp, extract_media_id

app = typer.Typer(help="导出B站收藏夹视频列表到CSV")


@app.command(name="export")
def export_command(
    fav_url: str = typer.Argument(..., help="B站收藏夹页面URL"),
    output: Path = typer.Option(Path("favlist.csv"), "--output", "-o", help="输出CSV路径"),
    encoding: str = typer.Option("gbk", "--encoding", "-e", help="输出文件编码"),
    page_size: int = typer.Option(40, "--page-size", help="单次请求条目数，默认40"),
    cookie: Optional[str] = typer.Option(None, "--cookie", help="必要时附加的Cookie"),
    timeout: float = typer.Option(10.0, "--timeout", help="请求超时时间(秒)"),
) -> None:
    """导出收藏夹条目。"""
    if page_size <= 0:
        raise typer.BadParameter("page_size必须大于0")
    typer.echo("解析收藏夹链接...")
    try:
        media_id = extract_media_id(fav_url)
    except ValueError as exc:
        raise typer.BadParameter(str(exc)) from exc

    session = build_session(cookie=cookie)
    client = BiliFavClient(session=session, timeout=timeout)

    try:
        folder_info = client.get_folder_info(media_id)
    except (BiliAPIError, BiliRequestError) as exc:
        typer.secho(f"获取收藏夹信息失败: {exc}", fg=typer.colors.RED)
        raise typer.Exit(code=1) from exc

    typer.echo(f"收藏夹：{folder_info.title} (共{folder_info.media_count}个条目，media_id={folder_info.media_id})")

    typer.echo("读取历史CSV...")
    try:
        existing_ids = load_existing_bv_ids(output, encoding)
    except ValueError as exc:
        typer.secho(str(exc), fg=typer.colors.RED)
        raise typer.Exit(code=1) from exc

    typer.echo("开始抓取收藏夹条目...")
    try:
        medias = list(client.iter_videos(media_id, page_size=page_size))
    except (BiliAPIError, BiliRequestError) as exc:
        typer.secho(f"抓取收藏夹条目失败: {exc}", fg=typer.colors.RED)
        raise typer.Exit(code=1) from exc

    timestamp = current_timestamp()
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
                timestamp=timestamp,
                aid=item.get("id"),
            )
        )

    if not new_entries:
        typer.echo("没有新的条目需要写入。")
        raise typer.Exit(code=0)

    written = write_entries(output, encoding, new_entries)
    typer.echo(f"写入完成，新增加 {written} 条记录，输出文件：{output}")
    raise typer.Exit(code=0)


def main() -> None:
    app()


if __name__ == "__main__":
    main()
