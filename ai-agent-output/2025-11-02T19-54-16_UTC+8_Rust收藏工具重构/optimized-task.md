# 优化后的任务描述

## 背景与目标
- Python 版本的 `auto_download_favlist` 与 `bilibili_favlist_download_helper` 已迁移至 `py-src/`，用于参考历史行为。
- 现需以 Rust 重写两套能力：收藏夹导出工具与交互式下载助手，形成同仓库内的 Rust 工作区。
- Rust 版本必须通过共享 crate 复用业务能力，而非通过 Shell 调用二进制程序；交互式助手只能使用方向键/WASD、Enter/Space、Esc 完成交互。

## 交付物清单
1. `Cargo.toml` 工作区配置与 `crates/` 目录下的新建 Rust crate：
   - `favlist_core`：库 crate，封装收藏夹抓取、模型、CSV 读写、错误类型、去重逻辑。
   - `get_bilibili_favlist_bvid_list`：二进制 crate，复刻导出 CLI，支持编码参数、增量去重、友好错误提示。
   - `bilibili_favlist_download_helper`：二进制 crate，提供交互式菜单并直接调用 `favlist_core` 能力。
2. 对应单元/集成测试：HTTP 客户端抽象（可 mock）、CSV 去重、配置存储、菜单输入状态机等。
3. PlantUML 源文件与导出的 PNG 流程图（覆盖两个程序的主流程）。
4. 更新的 `optimized-task.md`、`undetermined.md`、`AI-AGENT_working-status.csv`、流程设计文档与其它必要文档（如 README/CHANGELOG）。

## 执行步骤概览
1. **工作区初始化**：创建 `Cargo.toml`（workspace）、`.cargo/config.toml`（若需），建立 `crates/` 目录与基础模块骨架。
2. **核心逻辑迁移**：在 `favlist_core` 中实现：
   - HTTP 客户端（`reqwest` + `tokio`）封装、收藏夹分页抓取、数据模型与错误类型。
   - CSV 读写、编码处理（UTF-8/GBK 等）、增量去重与时间戳工具。
   - 对外暴露同步友好 API（包装异步执行），供二进制 crate 调用。
3. **导出 CLI 实现**：
   - 在 `get_bilibili_favlist_bvid_list` 中构建 clap 命令参数，支持输出路径/编码/page_size/cookie/timeout。
   - 处理既有 CSV 去重逻辑，输出提示与错误颜色，为零新增场景返回退出码 0。
4. **交互式助手实现**：
   - 编写配置存储（XDG/Windows 目录）与模型序列化（JSON）。
   - 导入 `favlist_core` 功能执行导出、差异比较、缺漏检测（需要目录扫描、bbdown 命令包装）。
   - 基于 `crossterm`/`ratatui` 实现菜单状态机，仅接受方向键/WASD 与 Enter/Space/ESC。
   - 提供“录入新收藏夹”“使用存档配置→编辑/检查更新/检查缺漏”等流程，dry-run 模式下仅打印命令。
5. **测试与验证**：
   - 使用 mock HTTP（`httpmock` 或 `wiremock-rs`）验证分页与错误处理。
   - 针对 CSV 去重、文件系统操作、菜单状态管理编写单元测试；对关键流程提供集成测试或脚本验证。
   - 运行 cargo fmt/clippy/test，收集关键输出。
6. **文档与收尾**：
   - 输出流程设计文档与 PlantUML + PNG。
   - 更新 `README.md`、`CHANGELOG.md`，说明 Rust 版本使用方法与迁移注意事项。
   - 根据自检表逐项确认并整理任务目录，重置 `ai-agent-memory/now-task.md`。

## 验证标准
- `cargo test`、`cargo clippy`、`cargo fmt` 全部通过；关键模块具备针对性测试。
- `get_bilibili_favlist_bvid_list` 能成功导出示例收藏夹，并在重复执行时保持去重；错误场景提供友好输出与正确退出码。
- `bilibili_favlist_download_helper` 菜单交互仅响应方向键/WASD + Enter/Space/ESC，dry-run 模式打印 `bbdown` 命令；真实模式可检测命令缺失并提示。
- 配置文件、CSV、备份与目录清单生成位置与 Python 版一致或在文档中说明差异。
- PlantUML PNG 与文档同步，`CHANGELOG.md` 新增版本段落并保留 `[Unreleased]`。

## 关键风险与应对
- **B站接口变动/限流**：在核心 crate 中统一封装错误并提供重试/间隔配置，文档提示用户准备 Cookie。
- **编码兼容性**：引入 `encoding_rs` + `encoding_rs_io` 处理 GBK/UTF-8，测试中覆盖双编码场景。
- **交互复杂度**：菜单状态机与按键监听通过单元测试覆盖，提供降级方案（在文档中说明若终端不可用可退回 Python 版）。
- **依赖执行环境**：`bbdown` 缺失时捕获错误并提示安装，引导使用 `--dry-run`。

## 后续记录约定
- 所有状态、决策、风险在 `AI-AGENT_working-status.csv`、`optimized-task.md` 或设计文档中同步。
- 用户要求无需额外确认，返工时需在状态文件中记录原因与回退环节。
