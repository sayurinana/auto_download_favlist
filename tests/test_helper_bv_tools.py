from pathlib import Path

from bilibili_favlist_download_helper import bv_tools


def test_scan_directory_and_inventory(tmp_path: Path) -> None:
    (tmp_path / "videos").mkdir()
    file_with_bv = tmp_path / "videos" / "示例-BV1ABCDEF123.mp4"
    file_with_bv.write_text("test")
    file_without_bv = tmp_path / "videos" / "no_bv.mp4"
    file_without_bv.write_text("test")

    mapping = bv_tools.scan_directory_bvids(tmp_path)
    assert "BV1ABCDEF123" in mapping
    assert file_with_bv in mapping["BV1ABCDEF123"]

    inventory_path = bv_tools.write_inventory_file(tmp_path, mapping)
    assert inventory_path.exists()
    content = inventory_path.read_text(encoding="utf-8")
    assert "BV1ABCDEF123" in content


def test_find_missing_videos(tmp_path: Path) -> None:
    csv_path = tmp_path / "fav.csv"
    csv_path.write_text(
        "timestamp,bv_id,title,fav_name\n"
        "2025-01-01T00-00-00,BV1ABCDEF123,视频A,收藏夹\n"
        "2025-01-01T00-00-01,BV1GHIJKL456,视频B,收藏夹\n",
        encoding="utf-8",
    )
    rows = bv_tools.read_csv_rows(csv_path)
    mapping = {"BV1ABCDEF123": [Path("dummy.mp4")]}  # 只有一个视频存在
    missing = bv_tools.find_missing_videos(rows, mapping.keys())
    assert len(missing) == 1
    assert missing[0]["bv_id"] == "BV1GHIJKL456"
