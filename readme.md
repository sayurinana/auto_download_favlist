# auto_download_favlist

B站收藏夹批量导出工具，基于Python构建。支持通过CLI传入收藏夹URL，抓取分页视频列表并转换为CSV。现新增交互式助手，可管理多条收藏夹配置并自动触发 `bbdown` 下载。

## 快速上手
1. 安装依赖：
   ```bash
   uv venv
   uv pip install -r requirements.txt
   ```
2. 执行导出：
   ```bash
   PYTHONPATH=src uv run python -m auto_download_favlist.cli "<favlist_url>" --output output/favlist.csv
   ```

3. 交互式助手：
   ```bash
   PYTHONPATH=src uv run python -m bilibili_favlist_download_helper --dry-run
   ```
   首次运行可录入收藏夹 URL 与下载目录，后续可通过“检查更新”“检查缺漏”复用历史配置。移除 `--dry-run` 后即会实际调用 `bbdown`。

更多平台部署与参数说明请参考`docs/windows-deployment.md`。
