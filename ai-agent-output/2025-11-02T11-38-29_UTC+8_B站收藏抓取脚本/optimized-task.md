# 优化后的任务描述

## 背景与目标
- 构建可在Linux与Windows运行的Python爬虫/CLI工具，接收B站收藏夹链接并批量导出收藏内容。
- 默认针对示例收藏夹 `https://space.bilibili.com/234561771/favlist?fid=3670113371` 进行开发与回归测试。
- 输出包含时间戳、BV号、视频标题、收藏夹名的CSV文件，默认GBK编码，允许通过参数覆盖。

## 交付物清单
1. `src/` 下的Python实现（包含CLI入口与抓取逻辑）。
2. 单元或集成测试覆盖核心流程（分页抓取、去重、编码切换）。
3. Windows部署与使用指南文档（含依赖安装、运行示例、常见问题）。
4. 任务流程PlantUML源码与导出的PNG流程图。
5. 更新后的`optimized-task.md`、`undetermined.md`、`AI-AGENT_working-status.csv`记录。

## 执行步骤概览
1. **流程设计**：
   - 产出`flow-design.md`，说明模块划分、依赖、风险及降级方案。
   - 编写PlantUML描述流程，导出PNG并存档。
2. **实现阶段**：
   - 实现CLI（基于Typer）与抓取服务模块，封装B站API调用、分页、数据清洗、CSV写入。
   - 支持可选参数：`--output`、`--encoding`、`--cookie`/`--headers-json`等。
   - 去重策略：读取既有CSV构建已存BV集合，新数据过滤后再写入。
3. **验证阶段**：
   - 针对示例收藏夹运行脚本，确认生成CSV且无重复条目。
   - 编写并执行测试（若接口需mock，使用`pytest`+`responses`或`pytest-httpx`）。
   - 记录测试命令与结果。
4. **文档与收尾**：
   - 更新`README`或新增`docs/windows-usage.md`说明部署与运行。
   - 维护`CHANGELOG.md`新版本段落与空`[Unreleased]`区块。
   - 完成执行自检表并在`AI-AGENT_working-status.csv`记录结论。

## 验证标准
- CLI命令成功抓取全部分页数据，字段完整且数量与收藏夹统计一致。
- CSV写入符合指定编码，重复执行不会写入重复BV号。
- 流程图与文档与实现一致，可指导Windows端用户部署运行。
- 代码通过自建测试并提供运行日志/截图证明。

## 执行与验证纪要
- 2025-11-02 在uv虚拟环境中运行`pytest`，通过`tests/test_cli.py`、`tests/test_csv_writer.py`、`tests/test_utils.py`共5项用例。
- 同日使用`PYTHONPATH=src uv run python -m auto_download_favlist.cli`对示例收藏夹抓取，生成`output/favlist.csv`共109条新纪录，确认分页与去重逻辑有效。
- `requirements.txt`固定`click>=8.1,<8.2`防止Typer兼容性问题；后续在Windows上沿用`uv pip install -r requirements.txt`即可复现环境。

## 关键约束与说明
- 依据用户指示“无需与用户沟通确认”，所有待定项已由代理自决并记录在`undetermined.md`。
- 必须遵循`AI-AGENT_criterion.md`的六环节流程与提交规范，所有状态变更同步进CSV并即时提交。
- 推荐通过`uv venv`创建虚拟环境，确保跨平台一致性；如需外部依赖，保证Windows兼容。
- 若B站API需认证信息，需在文档中说明如何获取并传入（同时避免泄露敏感数据）。
