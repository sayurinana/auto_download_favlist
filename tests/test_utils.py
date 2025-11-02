import re

import pytest

from auto_download_favlist.utils import current_timestamp, extract_media_id


def test_extract_media_id_success() -> None:
    url = "https://space.bilibili.com/234561771/favlist?fid=3670113371"
    assert extract_media_id(url) == 3670113371


def test_extract_media_id_failure() -> None:
    with pytest.raises(ValueError):
        extract_media_id("https://space.bilibili.com/234561771")


def test_current_timestamp_format() -> None:
    timestamp = current_timestamp()
    assert re.match(r"^\d{4}-\d{2}-\d{2}T\d{2}-\d{2}-\d{2}$", timestamp)
