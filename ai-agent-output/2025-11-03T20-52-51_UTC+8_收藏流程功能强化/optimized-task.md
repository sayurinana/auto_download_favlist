# 优化后的任务描述

## 背景与目标
- 现有 Rust 工作区已提供 `favlist_core`、`get_bilibili_favlist_bvid_list` 与 `bilibili_favlist_download_helper` 三个 crate，实现收藏夹导出、CLI 与交互式助手能力。
- 新需求聚焦于界面体验提升、抓取进度可视化，以及缺漏补全流程对接 `BBDown serve` JSON API，兼顾 dry-run 场景。
- 同时需支持在助手配置中维护 `file_pattern`、`multi_file_pattern`，以便通过 API 动态拼装完整 `FilePattern` 字段而无需手工传参。

## 交付物清单
1. `favlist_core`
   - 新增进度回调机制，导出流程在抓取/写入时反馈“当前数量/总量”数据。
   - 扩展导出接口返回文件总量、页面信息，兼容既有调用者。
2. `get_bilibili_favlist_bvid_list`
   - 使用 `console`/`indicatif` 提升输出样式，展示抓取进度条与彩色提示。
   - 适配核心库进度回调，提供终端友好的“已抓取/总数”展示。
3. `bilibili_favlist_download_helper`
   - 更新 `FavConfig` 与配置存储：新增 `bbdown_serve_url`、`bbdown_auto_launch`、`bbdown_launch_args`、`bbdown_poll_interval_ms`、`file_pattern`、`multi_file_pattern` 等字段，并提供迁移兼容逻辑。
   - 在“检查缺漏”流程中，依据配置决定是否启动 `BBDown serve` 子进程；dry-run 时仅打印命令。
   - 封装 `bbdown` API 客户端，支持添加任务、轮询运行/完成列表、在空闲时关闭进程并再次检查缺漏。
   - 生成 `FilePattern` 字段时，将下载目录与 pattern 正确拼接并处理 Windows 反斜杠转义。
   - 交互界面统一使用 `console::style`、`console::Emoji`、`indicatif` 等提供彩色提示与进度反馈。
4. 文档与支撑资料
   - 更新 `optimized-task.md`、`undetermined.md`、`AI-AGENT_working-status.csv`、流程设计文档、流程图 PNG。
   - 更新 `CHANGELOG.md` 新增版本段落并重建 `[Unreleased]`。
   - 必要时补充 README/配置说明，解释新字段与 API 调用行为。

## 执行步骤概览
1. **任务分析落地**
   - 根据待定项确定依赖 crate (`console`, `indicatif`, `reqwest` blocking 客户端等) 与配置结构调整。
2. **核心库增强**
   - 在 `favlist_core::export` 中加入进度回调接口；包装同步/异步导出函数兼容默认行为。
   - 提供事件类型（如 `ExportProgress { current, total }`）并通过 `ExportOptions` 接收回调。
   - 扩展单元测试验证进度回调触发次数与顺序（可通过 mock 客户端）。
3. **CLI 进度整合**
   - 在 `get_bilibili_favlist_bvid_list` 中创建 `ProgressBar`，当总量未知时退回 spinner。
   - 提升输出信息色彩与结构，保持错误处理一致。
4. **助手配置与BBDown API集成**
   - 迁移 `FavConfig` 结构与存储，确保旧配置加载时填充默认值。
   - 新增 `bbdown::api` 模块封装 `/add-task`、`/get-tasks/running`、`/get-tasks/finished`、`/remove-finished` 等方法，支持自定义请求间隔。
   - 在缺漏流程中，依据 missing 列表批量提交任务，随后轮询直至运行任务为空；dry-run 模式打印计划操作。
   - 任务完成后再执行一次缺漏检查，但不重复下发任务，输出最终结果汇总。
   - 视情况在 `menu` 模块增加彩色提示与错误高亮。
5. **子进程管理与资源清理**
   - 若启用自动启动 `BBDown serve`，使用 `Command` 启动并在完成或出现错误时关闭。
   - 对于 dry-run 或不启动场景，打印说明并跳过关闭逻辑。
6. **测试与验证**
   - 为配置迁移、进度回调、API 客户端、FilePattern 拼接逻辑编写单元测试。
   - 在 dry-run 与实际请求（可通过 httpmock 模拟）之间进行集成测试。
   - 运行 `cargo fmt`, `cargo clippy --all-targets --all-features -D warnings`, `cargo test` 确认通过。
7. **文档与流程图更新**
   - 在流程设计与 PlantUML 中反映新增步骤（进度回调、BBDown API 轮询等）。
   - 更新 README/CHANGELOG 等，说明新配置项与使用方式。

## 验证标准
- 所有新增/修改 crate 均能编译通过，测试覆盖进度回调、API 调度与 FilePattern 拼接关键路径。
- `get_bilibili_favlist_bvid_list` 在导出收藏夹时显示“当前/总数”进度条，并在完成后清空进度。
- `bilibili_favlist_download_helper`：
  - 能加载旧版配置并自动补全默认字段；编辑界面可维护新模式字段。
  - 检查缺漏时根据配置自动启动/跳过 `BBDown serve`，dry-run 仅输出操作说明。
  - 能通过 API 添加任务并在所有任务完成后退出轮询，输出最终缺漏结果。
- `FilePattern` 字段字符串符合示例格式，Windows 与 Unix 环境均正确拼接路径分隔符。
- 文档、流程图、CHANGELOG、状态记录同步更新，自检表项均完成。

## 关键风险与应对
- **进度总数无法即时获取**：若接口未返回总数，提供 fallback spinner 并在日志中说明。
- **BBDown API 不可用或返回异常**：为 API 客户端加入超时/重试与错误提示，必要时回退至 dry-run 提醒用户。
- **配置迁移兼容性**：为缺失字段设置默认值，并在存储时自动写回；提供测试覆盖旧配置 JSON。
- **路径转义差异**：针对 Windows 反斜杠使用 `replace("\\", "\\\\")` 统一处理，测试验证示例路径。

## 备注
- 用户已声明无需逐项确认待定项，本任务按既定方案自主推进；如需返工，将在状态记录中说明原因与回退环节。