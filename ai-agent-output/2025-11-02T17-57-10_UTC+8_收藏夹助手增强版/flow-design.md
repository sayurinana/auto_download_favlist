# 流程设计

## 模块划分
1. `helper.config_store`：读取/写入配置 JSON，定义 `FavlistConfig` 数据类与仓库接口。
2. `helper.export_service`：封装收藏夹抓取逻辑，生成时间戳 CSV 并返回新条目列表。
3. `helper.bv_tools`：提供 BV 号解析、CSV 差异对比、目录扫描工具函数。
4. `helper.cli`：Typer 应用主入口，实现交互式菜单与子流程。
5. `helper.bbdown_runner`：统一封装 `bbdown` 调用，支持 `dry_run`。

## 数据流
- 用户通过 CLI 选择操作；
- 配置仓库负责加载/更新记录；
- 录入流程调用 `export_service` 生成 CSV，更新配置并写回；
- 检查更新流程先备份旧 CSV，再调用 `export_service` 获取新数据，使用 `bv_tools` 比对新增条目并触发下载；
- 检查缺漏流程使用 `bv_tools` 扫描目录 + CSV 对比，缺失项调用 `bbdown_runner`。

## 执行步骤细化
1. **配置仓库**
   - 数据类字段：`name`（可选描述）、`fav_url`、`download_dir`、`csv_path`、`last_synced_at`。
   - 方法：`load_all()`, `save_all()`, `upsert(config)`, `select(index)`。
2. **导出服务**
   - 封装 `auto_download_favlist` 逻辑；遇到 `typer.Exit`、`SystemExit` 转换为自定义异常。
   - 生成 CSV 文件名：`{timestamp}-favlist.csv`。
3. **CLI 流程**
   - 主菜单：`录入新收藏夹 / 使用存档 / 退出 / (dry-run显示)`。
   - 录入：提示 URL、目录；创建 CSV；写入配置。
   - 存档操作：列出配置 -> 选择 -> 动作（编辑/检查更新/检查缺漏/返回）。
   - 编辑流程：逐项提示新值，可回车保留；自动刷新 CSV 路径与更新时间（若重新抓取）。
4. **检查更新**
   - 旧 CSV 重命名为 `*.backup.csv`。
   - 运行抓取，得到新 CSV。
   - 使用 `bv_tools.diff_bvids(old_csv, new_csv)` 输出新增条目。
   - 遍历新增条目，调用 `bbdown_runner.run(bv_id, download_dir, dry_run)`。
5. **检查缺漏**
   - 扫描目录所有文件名，正则提取 BV 号并建立集合。
   - 读取当前 CSV 的 `bv_id` 列，找出缺失。
   - 对缺失列表调用 `bbdown_runner`。
6. **异常处理与日志**
   - 所有外部调用捕获异常并 `typer.secho` 提示。
   - 关键操作输出成功信息与下一步建议。

## 风险与回退策略
- **抓取失败**：保留备份 CSV，提示用户稍后重试；失败不覆盖配置。
- **配置写入失败**：输出错误并中止，保留旧配置。
- **bbdown 不存在**：提示安装并继续下一个条目。

## 验证计划
- CLI 干跑 `--dry-run` 演练核心路径。
- 针对 `bv_tools` 与配置仓库编写单元测试。
- 测试 CSV 差异、缺漏识别与命令拼接。
