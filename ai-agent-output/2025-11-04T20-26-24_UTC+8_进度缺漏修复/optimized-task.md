# 优化后的任务描述

## 背景
- `bilibili_favlist_download_helper` 已具备进度回调、BBDown API 集成功能，但在真实环境（Windows BBDown + WSL CLI）中暴露出任务状态误判、缺少抓取进度、目录路径不一致等问题。
- 用户在 Windows 中运行 `bbdown serve`，WSL 仅负责抓取与缺漏检测，因此需要细化配置与流程，确保 API 调度与本地目录扫描能够分离工作路径。

## 目标
1. **修复任务状态判定 bug**：提交下载任务后准确侦测任务完成，避免提前结束提示。
2. **完善抓取进度反馈**：在助手的“录入新收藏夹”和“检查更新”流程中展示 `已获取/总数` 的实时进度信息。
3. **改进 `bbdown serve` 启动体验**：当需要自动启动时，在 Windows 中以独立控制台窗口运行 `bbdown serve`。
4. **分离 API 与扫描目录**：可为 BBDown API 提供 Windows 路径，同时在 WSL 使用对应的本地挂载路径进行缺漏检测。
5. **支持全局默认配置**：允许在全局配置中维护默认下载目录、BBDown URL、File Pattern/Multi File Pattern，录入新收藏夹时可直接复用。
6. **实测“测试用”配置**：使用 `.config/bilibili_favlist_helper/config.json` 中的“测试用”项进行实际抓取与缺漏补全验证（非 dry-run）。

## 交付物
- 代码：`favlist_core`（若需调整回调）、`bilibili_favlist_download_helper`（config、bbdown、main 模块）以及可能的新 `global_config` 读写模块。
- 配置示例与迁移逻辑：全局默认文件示例、现有配置升级说明。
- 测试：
  - 单元/集成测试覆盖任务轮询逻辑（可通过 httpmock 模拟 Running/Finished）。
  - CLI 手动测试日志：使用 `fid=3685051971` 抓取展示进度；执行“测试用”配置补缺流程，附关键输出。
- 文档：更新 `CHANGELOG.md`、`optimized-task.md`、`AI-AGENT_working-status.csv` 等记录。

## 执行步骤
1. **调研现状**：
   - 复查 `bbdown.rs` API 客户端、`check_missing` 流程以及现有配置结构。
   - 梳理 `.config/bilibili_favlist_helper/config.json` 内容，确认“测试用”项。
2. **设计方案**：
   - 确定全局默认配置文件路径、字段以及读取优先级（全局 -> 配置文件 -> 命令输入）。
   - 明确 API 目录与扫描目录的字段命名与 UI 提示。
   - 规划任务轮询逻辑（记录提交 BV / 监控 running 与 finished 列表 / 超时提示）。
3. **实现改动**：
   - 扩展 `FavConfig` 结构、序列化与迁移；新增全局默认读写模块。
   - 更新录入/编辑流程，提供默认值提示与新字段输入；更新 `check_update` 和 `check_missing` 对应逻辑。
   - 调整 `bbdown` 模块，支持 Windows 独立窗口启动与更严格的任务轮询。
   - 接入进度回调，展示 `indicatif` 进度条。
4. **实测与验证**：
   - 删除 `/mnt/d/download-buffer/BBDown/star/test-1/` 中的下载文件后，使用“测试用”配置执行“检查缺漏”，观察任务提交、轮询与目录复检输出。
   - 使用 fid=3685051971 执行一次抓取，确认进度条与统计信息。
   - 运行 `cargo fmt`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`。
5. **文档与交付**：
   - 更新 `CHANGELOG.md`、`optimized-task.md`、流程记录与若需的 README 片段。
   - 在 `AI-AGENT_working-status.csv` 填写阶段信息，并整理验证结果/遗留风险。

## 验证标准
- 任务轮询：连续轮询到 Running 为空且指定 BV 进入 finished，再次扫描下载目录时若仍缺失需继续告警而非直接完成。
- 抓取进度：在助手 UI 中可实时看到 `已获取/总数`，收尾后输出处理条数与 CSV 路径。
- Windows 启动：当 `bbdown_auto_launch` 为 true 且检测到 WSL/Windows，Command 行为变为调用 `powershell.exe Start-Process`（即使当前环境无法真正弹窗，也应打印所执行命令）。
- 目录分离：API 请求 payload 中使用 Windows 路径，扫描逻辑使用 WSL 路径，二者可独立配置。
- 全局默认：新增/编辑配置时未输入值则使用全局默认，并在提示中展示默认值；全局文件缺失时不影响现有功能。
- 实测日志：输出“测试用”配置一次完整补缺流程的关键日志（含任务提交数量、轮询摘要、再次扫描结果）。
