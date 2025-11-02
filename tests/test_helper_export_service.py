from pathlib import Path

import responses

from bilibili_favlist_download_helper.export_service import ExportResult, export_to_csv
from auto_download_favlist.fav_client import INFO_ENDPOINT, LIST_ENDPOINT


def _mock_folder_response(media_id: int) -> dict:
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


def _mock_list_response(media_id: int, bv_id: str) -> dict:
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
def test_export_to_csv_writes_new_entries(tmp_path: Path) -> None:
    media_id = 3546830971
    responses.add(responses.GET, INFO_ENDPOINT, json=_mock_folder_response(media_id), status=200)
    responses.add(responses.GET, LIST_ENDPOINT, json=_mock_list_response(media_id, "BV1ABCDEF123"), status=200)

    fav_url = f"https://space.bilibili.com/234561771/favlist?fid={media_id}"
    csv_path = tmp_path / "fav.csv"
    result = export_to_csv(fav_url, csv_path, timestamp="2025-01-01T00-00-00")

    assert isinstance(result, ExportResult)
    assert csv_path.exists()
    content = csv_path.read_text(encoding="utf-8").splitlines()
    assert len(content) == 2
    assert "BV1ABCDEF123" in content[1]
    assert result.new_entries and result.new_entries[0].bv_id == "BV1ABCDEF123"


@responses.activate
def test_export_to_csv_skips_existing_entries(tmp_path: Path) -> None:
    media_id = 3546830971
    responses.add(responses.GET, INFO_ENDPOINT, json=_mock_folder_response(media_id), status=200)
    responses.add(responses.GET, LIST_ENDPOINT, json=_mock_list_response(media_id, "BV1ABCDEF123"), status=200)

    fav_url = f"https://space.bilibili.com/234561771/favlist?fid={media_id}"
    csv_path = tmp_path / "fav.csv"
    export_to_csv(fav_url, csv_path, timestamp="2025-01-01T00-00-00")

    responses.add(responses.GET, INFO_ENDPOINT, json=_mock_folder_response(media_id), status=200)
    responses.add(responses.GET, LIST_ENDPOINT, json=_mock_list_response(media_id, "BV1ABCDEF123"), status=200)

    result = export_to_csv(fav_url, csv_path, timestamp="2025-01-02T00-00-00")

    assert not result.new_entries
    content = csv_path.read_text(encoding="utf-8").splitlines()
    assert len(content) == 2
