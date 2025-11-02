from pathlib import Path

from bilibili_favlist_download_helper.config_store import ConfigRepository, FavlistConfig


def test_config_repository_roundtrip(tmp_path: Path) -> None:
    storage = tmp_path / "config.json"
    repo = ConfigRepository(storage)
    assert repo.is_empty()

    config = FavlistConfig(
        fav_url="https://example.com",
        download_dir=str(tmp_path),
        csv_path=str(tmp_path / "fav.csv"),
        name="演示配置",
    )
    repo.add(config)

    repo_reloaded = ConfigRepository(storage)
    assert not repo_reloaded.is_empty()
    loaded = repo_reloaded.get(0)
    assert loaded.fav_url == config.fav_url
    assert loaded.download_dir == config.download_dir
    assert loaded.name == "演示配置"


def test_config_repository_update_and_remove(tmp_path: Path) -> None:
    storage = tmp_path / "config.json"
    repo = ConfigRepository(storage)
    config = FavlistConfig(
        fav_url="https://example.com",
        download_dir=str(tmp_path),
        csv_path=str(tmp_path / "fav.csv"),
    )
    repo.add(config)

    updated = FavlistConfig(
        fav_url="https://example.com/new",
        download_dir=str(tmp_path / "downloads"),
        csv_path=str(tmp_path / "downloads" / "fav.csv"),
    )
    repo.update(0, updated)
    assert repo.get(0).fav_url == updated.fav_url

    removed = repo.remove(0)
    assert removed.fav_url == updated.fav_url
    assert repo.is_empty()
