# Agents.md

!!! 在开始任何工作前，必须先阅读本文件，并逐一查阅`ai-agent-memory/`目录中的所有指引。

## 快速指引
- `ai-agent-memory/AI-AGENT_criterion.md`：编程任务六环节流程、验证要点与 MCP 使用速览（含执行自检表）。
- `ai-agent-memory/AI-AGENT_step-before.md`：任务前置准备的顺序清单。
- `ai-agent-memory/undetermined-template.md`：待定项整理模板。
- `ai-agent-memory/now-task.md`：任务描述模板，执行前需完成内容预处理。

## 工作要求摘要
1. **工作状态同步**：首次进入“任务前置准备”环节时创建`AI-AGENT_working-status.csv`，并将`AI-AGENT_working-status.csv`、`undetermined.md`、`optimized-task.md`等辅助文档统一存放在`ai-agent-output/YYYY-MM-DDTHH-MM-SS_UTC+8_任务简述/`目录下（任务简述控制在8-20字），按环节维护记录并在每次更新该文件后立即创建git提交以保留状态变更轨迹。
2. **流程驱动执行**：严格遵循“任务前置准备 → 流程设计 → 任务主体流程循环 → 验证结果 → 文档更新 → 收尾”的顺序执行，返工时需回退环节并记录原因。
3. **信息留痕**：所有阶段性成果、关键决策、风险与约定必须记录在相应文档中（含本文件、流程设计文档、`optimized-task.md`等）。
4. **交付完成标准**：以`AI-AGENT_criterion.md`末尾的执行自检表逐环节核对，验证交付物满足成功标准后方可收尾。
5. **提交顺序要求**：每次准备提交前先运行`git add .`以追踪所有未被`.gitignore`忽略的文件；在“文档更新”环节更新工作状态后，务必先维护`CHANGELOG.md`并生成新的版本号段落（写入日期、重建空的`[Unreleased]`区块），然后创建提交。

如需新增或调整项目约定，请在完成任务后同步更新本文件与模板库中的原始文档。
