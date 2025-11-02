from pathlib import Path

import responses
from typer.testing import CliRunner

from auto_download_favlist import cli as cli_module
from auto_download_favlist.cli import app
from auto_download_favlist.fav_client import INFO_ENDPOINT, LIST_ENDPOINT


def mock_folder_response(media_id: int) -> dict:
    return {
        "code": 0,
        "message": "0",
        "data": {
            "id": media_id,
            "fid": media_id,
            "mid": 234561771,
            "title": "测试收藏夹",
            "media_count": 1,
        },
    }


def mock_list_response(media_id: int, bv_id: str) -> dict:
    return {
        "code": 0,
        "message": "0",
        "data": {
            "info": {
                "id": media_id,
                "title": "测试收藏夹",
            },
            "medias": [
                {
                    "id": 123,
                    "title": "演示视频",
                    "bv_id": bv_id,
                }
            ],
            "has_more": False,
        },
    }


@responses.activate
def test_cli_export_and_deduplicate(tmp_path: Path) -> None:
    media_id = 3670113371
    responses.add(
        responses.GET,
        INFO_ENDPOINT,
        json=mock_folder_response(media_id),
        status=200,
        repeat=2,
    )
    responses.add(
        responses.GET,
        LIST_ENDPOINT,
        json=mock_list_response(media_id, "BV123"),
        status=200,
        repeat=2,
    )

    original_ts = cli_module.current_timestamp
    cli_module.current_timestamp = lambda: "2025-01-01T00-00-00"  # type: ignore

    try:
        runner = CliRunner()
        output_csv = tmp_path / "fav.csv"
        result = runner.invoke(
            app,
            [
                "export",
                f"https://space.bilibili.com/234561771/favlist?fid={media_id}",
                "--output",
                str(output_csv),
                "--encoding",
                "utf-8",
            ],
        )
        assert result.exit_code == 0, result.stdout
        assert output_csv.exists()
        content = output_csv.read_text(encoding="utf-8").strip().splitlines()
        assert len(content) == 2  # header + one entry
        assert "BV123" in content[1]

        # 再次运行，应该提示无新增
        result_second = runner.invoke(
            app,
            [
                "export",
                f"https://space.bilibili.com/234561771/favlist?fid={media_id}",
                "--output",
                str(output_csv),
                "--encoding",
                "utf-8",
            ],
        )
        assert result_second.exit_code == 0, result_second.stdout
        assert "没有新的条目" in result_second.stdout
        content_after = output_csv.read_text(encoding="utf-8").strip().splitlines()
        assert len(content_after) == 2
    finally:
        cli_module.current_timestamp = original_ts  # type: ignore
