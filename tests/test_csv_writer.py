from pathlib import Path

from auto_download_favlist.csv_writer import load_existing_bv_ids, write_entries
from auto_download_favlist.models import VideoEntry


def test_write_and_load_csv(tmp_path: Path) -> None:
    csv_path = tmp_path / "fav.csv"
    entry = VideoEntry(
        bv_id="BV1abcd12345",
        title="示例视频",
        fav_title="收藏夹A",
        timestamp="2025-01-01T00-00-00",
    )

    written = write_entries(csv_path, "utf-8", [entry])
    assert written == 1
    assert csv_path.exists()

    existing = load_existing_bv_ids(csv_path, "utf-8")
    assert existing == {"BV1abcd12345"}

    # 再次写入不同条目
    new_entry = VideoEntry(
        bv_id="BV1efgh67890",
        title="示例视频2",
        fav_title="收藏夹A",
        timestamp="2025-01-02T00-00-00",
    )
    written_second = write_entries(csv_path, "utf-8", [new_entry])
    assert written_second == 1

    all_existing = load_existing_bv_ids(csv_path, "utf-8")
    assert all_existing == {"BV1abcd12345", "BV1efgh67890"}
