# auto_download_favlist

B站收藏夹批量导出工具，基于Python构建。支持通过CLI传入收藏夹URL，抓取分页视频列表并转换为CSV。

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

更多平台部署与参数说明请参考`docs/windows-deployment.md`。
