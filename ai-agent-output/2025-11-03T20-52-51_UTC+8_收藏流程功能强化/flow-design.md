# 流程设计

## 1. 总体架构
- 保持现有三 crate 架构：`favlist_core`（lib）、`get_bilibili_favlist_bvid_list`（bin）、`bilibili_favlist_download_helper`（bin）。
- 新增/调整模块：
  - `favlist_core::export`：引入 `ExportProgress` 事件、进度回调与总量统计；支持自定义基地址/请求头沿用现有接口。
  - `favlist_core::client`：补充获取总条目数逻辑，必要时返回 `Option<u64>`。
  - `bilibili_favlist_download_helper::bbdown::api`（新模块）：封装 HTTP API 调用。
  - `bilibili_favlist_download_helper::config`：扩展配置结构、提供迁移兼容与默认填充。
  - `bilibili_favlist_download_helper::workflow`（新增文件）：聚合“检查缺漏→API提交→轮询→收尾”流程，保持 `main.rs` 精简。
- 第三方依赖：
  - `indicatif`（CLI/助手进度条）。
  - `console`（彩色输出/emoji）。
  - `reqwest` blocking client 用于 API 调用（可重用 tokio runtime，或者使用 blocking API）。

## 2. 关键流程
1. **收藏夹导出进度回调**
   - `export_favlist` 在页级/条目级更新 `current`；若接口返回 `media_count`，作为 `total`。
   - 回调通过 `ExportOptions::progress_callback`（`Option<Arc<dyn Fn(ExportProgress)>>`）触发。
   - 写入 CSV 时保持与原逻辑一致。

2. **CLI进度展示与界面美化**
   - 初始化 `indicatif::ProgressBar`；若 `total` 不可用使用 spinner。
   - 每次回调更新进度、显示彩色文本（成功/警告/错误使用 `console::style`）。
   - 完成后清空进度并输出总结。

3. **助手配置扩展**
   - `FavConfig` 新增字段：
     - `bbdown_serve_url`（默认 `http://localhost:23333`）。
     - `bbdown_auto_launch`（bool, 默认 true）。
     - `bbdown_launch_args`（Vec<String>）。
     - `bbdown_poll_interval_ms`（默认 500）。
     - `file_pattern`、`multi_file_pattern`（Option<String>）。
   - 加载旧配置时填充默认值；保存时写回新字段。
   - 菜单编辑页支持调整上述字段。

4. **缺漏检查 + BBDown API 集成**
   - 启动阶段：若 `bbdown_auto_launch && !dry_run`，尝试 `Command::new("bbdown").args(["serve", ...])`。
   - 为缺漏列表批量生成 API 请求体（包含拼接后的 `FilePattern` 与可选 `MultiFilePattern`）。
   - 使用 `reqwest::blocking::Client` 调用 `/add-task`，记录失败项。
   - 轮询 `/get-tasks/running`，间隔 `poll_interval` 毫秒；若空 → 调用 `/remove-finished` → 关闭子进程。
   - 再次执行 `find_missing_videos`，仅输出结果，不再重复提交。
   - dry-run 模式：打印将要执行的 curl/JSON，不真正发起请求或启动服务。

5. **流程图**
   - 在 `program_flowchart/src` 新增 2 个 PlantUML：`core_export_progress.txt`（导出进度）与 `helper_bbdown_api.txt`（缺漏补全流程）。
   - 使用指定 PlantUML 命令生成 PNG。

## 3. 数据与文件
- 配置文件：`~/.config/bilibili_favlist_helper/config.json`，旧字段保留，新字段添加默认值。
- FilePattern 拼接：以配置下载目录 `download_dir` + `std::path::MAIN_SEPARATOR` + pattern；Windows 需要将反斜杠 double escape 在 JSON 中写入。
- 轮询日志写入终端，例如 `console::style("等待下载任务完成...").magenta()`。
- 轮询超时参数可考虑配置（暂使用轮询次数上限，如 240次=2分钟，可列入配置）。
- API 错误记录：集中输出到终端，并返回菜单以避免误操作。

## 4. 测试计划
- `favlist_core`
  - 新增单元测试：模拟两页数据，确认回调被调用且 `current/total` 正确。
  - 兼容总量缺失场景，确保 `total == None` 时回调行为得当。
- `get_bilibili_favlist_bvid_list`
  - CLI 单测：使用 `assert_cmd` 验证进度回调不会 panic，输出包含彩色标签（可通过去除 ANSI 测试）。
- `bilibili_favlist_download_helper`
  - 配置加载/保存测试：旧 JSON 自动填充新字段。
  - FilePattern 拼接测试：Windows/Unix 模式下路径转义正确。
  - BBDown API 客户端测试：使用 `httpmock` 模拟 `/add-task`、`/get-tasks/running`、`/remove-finished`。
  - 缺漏流程 dry-run 测试：确保无真实请求。
- 集成/验证
  - 运行 `cargo test --all`、`cargo clippy --all-targets --all-features -D warnings`、`cargo fmt`。

## 5. 里程碑拆分
1. 设计更新（当前阶段）→ 输出 flow-design、PlantUML 草稿。
2. 核心库：实现进度事件与测试。
3. CLI：进度条与彩色输出适配。
4. 助手配置扩展：结构、迁移、编辑 UI。
5. BBDown API 模块与缺漏流程重构；实现 dry-run。
6. 文档、流程图渲染、CHANGELOG 更新；执行自检。

## 6. 风险与预案
- **API 接口差异**：若 `media_count` 不准确，CLI 展示“未知总量”，并在日志标注。
- **BBDown serve 无法启动**：捕获错误，提示用户手动启动；若已配置远程URL，跳过子进程。
- **网络请求失败**：API 客户端加重试（简易：重试 3 次），失败收集至报告并继续流程。
- **路径转义错误**：提供单元测试覆盖示例路径；dry-run 输出最终 `FilePattern` 便于人工确认。
- **轮询阻塞体验差**：在等待期间输出剩余缺漏数量与提示用户可按 Ctrl+C 中断。