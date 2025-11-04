# 流程设计

## 1. 模块影响
- **配置层**：`config.rs` 需新增 API/扫描双目录、全局默认（独立文件）读写、迁移逻辑；录入/编辑界面需支持这些字段。
- **BBDown API**：`bbdown.rs` 扩展任务轮询（跟踪提交 BV / 检查 running+finished）、`start_bbdown_serve` 增加 Windows 独立窗口模式。
- **助手主流程**：`main.rs` 中的录入/检查更新/检查缺漏需接入进度条、引用新配置字段，并在“检查缺漏”中按 Windows/WSL 不同路径调用 API 与本地扫描。
- **核心库**：若需提供额外元数据（例如总量）已具备，不再改动。

## 2. 子步骤
1. **全局默认与配置结构**
   - 新增 `GlobalDefaults`（包含 download_dir_api、download_dir_scan、bbdown_url、file_pattern、multi_file_pattern）。
   - 程序启动时优先加载全局默认，再加载当前配置；录入流程在提示中展示默认值。
   - ConfigStore load 时对缺失字段赋默认值，兼容旧 JSON。
2. **路径分离实现**
   - `FavConfig` 添加 `api_download_dir`（Windows 路径）与 `scan_download_dir`（WSL 路径），提供解析方法。
   - 录入/编辑界面提示两套路径；若未填则沿用全局或历史值。
3. **任务轮询修复**
   - `BbdownApiClient` 在 `add_task` 返回 `Aid`（如 API 无返回则落回 URL），维护 `RunningTaskTracker`。
   - `wait_until_idle` 接收目标 Url/Aid 列表，直到运行列表中不存在目标任务且 finished 列表包含所有任务，或者达到超时给出提示。
4. **Windows 启动**
   - 在 WSL 中调用 `powershell.exe -Command Start-Process -FilePath "bbdown" -ArgumentList 'serve ...'`；其余平台仍 spawn 子进程。
5. **进度展示**
   - 录入新收藏夹/检查更新：创建 `ProgressBar` 并传入 `progress_callback`。
6. **测试 & 手动验证**
   - `httpmock` 单测覆盖：提交两个 BV，模拟 running→finished，确保等待逻辑正确。
   - 手动执行：
     - `cargo run -p bilibili_favlist_download_helper -- --config-path ~/.config/... --dry-run false` 使用“测试用”配置运行“检查缺漏”。
     - 运行抓取 fid=3685051971 观察进度条。
7. **文档/收尾**
   - 更新 `CHANGELOG.md`、默认配置说明、工作状态记录。

## 3. 时间/优先级
1. 配置/全局默认（约 1.5h）。
2. API & 轮询修复（约 2h，含测试）。
3. UI/交互调整（1h）。
4. 手动实测与日志整理（取决于网络，预估 1h）。
5. 文档与收尾（0.5h）。

## 4. 风险与应对
- **BBDown API 兼容性**：若 `/get-tasks/running` 不返回 Aid，则回退至 Url 匹配；若 API 不支持 finished 查询，则结合目录扫描来延长等待。
- **Windows 启动命令失败**：提供详细命令输出，并允许用户在配置中关闭自动启动。
- **全局文件缺失**：加载失败时写入默认模板，提示用户手工编辑。
- **真实测试依赖网络**：如连接失败，记录日志并说明需在具备网络时再验证。
