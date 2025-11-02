# auto_download_favlist

B 站收藏夹批量导出与下载辅助工具。项目已基于 Rust 重构核心抓取逻辑，并提供共享 crate 与两个命令行程序；原有 Python 实现仍保留，便于对照历史行为。

## Rust 版本快速上手
1. 安装 [Rust 工具链](https://www.rust-lang.org/tools/install)，确保具备 `cargo`。
2. 克隆仓库后执行收藏夹导出：
   ```bash
   cargo run -p get_bilibili_favlist_bvid_list -- \
     "https://space.bilibili.com/234561771/favlist?fid=3670113371" \
     --output output/favlist.csv --encoding gbk
   ```
   - 默认读取既有 CSV 并去重，可通过 `--cookie`、`--timeout`、`--page-size` 等参数调整。
3. 交互式助手（菜单仅响应方向键/WASD + Enter/Space/Esc）：
   ```bash
   cargo run -p bilibili_favlist_download_helper -- --dry-run
   ```
   - `--dry-run` 下仅打印 `bbdown` 命令；移除后会实际调用 `bbdown`。
   - 菜单支持录入收藏夹、编辑配置、检查更新/缺漏，并自动生成 CSV 备份与目录清单。

### crate 划分
- `favlist_core`：HTTP 客户端、数据模型、CSV 编解码与去重、目录盘点等共享能力。
- `get_bilibili_favlist_bvid_list`：单次导出命令行工具。
- `bilibili_favlist_download_helper`：交互式助手，通过 crate 复用导出逻辑并封装 `bbdown` 调度。

开发时建议执行：
```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Python 旧版（保留以供参考）
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

更多平台部署与参数说明请参考 `docs/windows-deployment.md`。Rust 与 Python 版本可按需选择使用。