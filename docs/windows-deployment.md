# Windows 部署与运行指引

## 1. 环境要求
- Windows 10 及以上版本，建议使用 PowerShell 7 或 Windows Terminal。
- 已安装 [Rust 工具链](https://www.rust-lang.org/tools/install)（用于运行/构建 Rust 版本）。
- 可选：已安装 [Python 3.11+](https://www.python.org/downloads/windows/) 与 [uv](https://docs.astral.sh/uv/getting-started/installation/)（便于运行旧版 Python 实现）。
- 可访问互联网以调用 B 站接口，必要时准备有效的账号 Cookie。

## 2. 获取代码
```powershell
# 建议在PowerShell中执行
cd C:\workspace
git clone https://example.com/auto_download_favlist.git
cd auto_download_favlist
```

## 3. 使用 Rust 版本
1. 运行收藏夹导出 CLI：
   ```powershell
   cargo run -p get_bilibili_favlist_bvid_list -- ^
     "https://space.bilibili.com/234561771/favlist?fid=3670113371" ^
     --output .\output\favlist.csv --encoding gbk
   ```
   - 如需传入 Cookie：追加 `--cookie "SESSDATA=xxxx; bili_jct=yyyy"`。
   - `--encoding` 默认为 `utf-8`，可按需切换至 `gbk`。
2. 交互式助手（方向键/WASD + Enter/Space/Esc 操作）：
   ```powershell
   cargo run -p bilibili_favlist_download_helper -- --dry-run
   ```
   - `--dry-run` 仅打印待执行的 `bbdown` 命令，移除后将真实下载。
   - 菜单可录入收藏夹、编辑配置、检查更新或缺漏，并生成 CSV 备份与目录清单。

## 4. 使用 Python 旧版（可选）
```powershell
uv venv
# PowerShell 激活
.\.venv\Scripts\Activate.ps1
```
若使用CMD：`\.venv\Scripts\activate.bat`。

## 4. 安装依赖
```powershell
uv pip install -r requirements.txt
```
> 依赖中锁定`click>=8.1,<8.2`以保持与Typer兼容；`uv`会自动解决余下依赖。

### 5. 运行示例
```powershell
$env:PYTHONPATH = "src"
uv run python -m auto_download_favlist.cli "https://space.bilibili.com/234561771/favlist?fid=3670113371" `
    --output .\output\favlist.csv `
    --encoding gbk
```
- 如需传入Cookie：加上`--cookie "SESSDATA=xxxx; bili_jct=yyyy"`。
- `--output`可指向自定义路径，默认编码GBK，亦可通过`--encoding utf-8`指定。

## 6. 常见问题
- **HTTP 412或412错误**：未设置`User-Agent`/`Referer`或Cookie失效，确认账号权限。
- **编码写入失败**：使用`--encoding utf-8`并确认目标文件支持该编码。
- **重复数据**：脚本会自动读取既有CSV并去重，如需刷新可删除旧文件后重跑。

## 7. 后续维护建议
- Rust 版本：可通过 `cargo update` 更新依赖，并执行 `cargo fmt && cargo clippy && cargo test` 验证。
- Python 版本：`uv pip install --upgrade -r requirements.txt` 可更新依赖。
- 结合 Windows 任务计划程序，可编写 `.ps1` 脚本按计划执行抓取。
- 若收藏夹为私密，需要登录后的 Cookie 方可访问，注意不要将 Cookie 写入公共仓库。
