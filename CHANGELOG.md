# Changelog

## [Unreleased]

## 2025-11-02 - 0.2.0
- 新增 `bilibili_favlist_download_helper` 交互式 CLI，支持录入/编辑配置、检查更新与缺漏、`--dry-run` 模式。
- 引入配置仓库、收藏夹导出封装、BV 解析工具与 `bbdown` 调度模块。
- 补充辅助流程文档及 PlantUML 流程图，完善任务跟踪文档。
- 增加针对助手模块的 pytest 覆盖，包含配置、导出与目录扫描测试。

## 2025-11-02 - 0.1.0
- 新增基于Typer的CLI工具，可抓取B站收藏夹并导出CSV。
- 引入去重、编码可配置以及B站API分页支持。
- 增加pytest单元/集成测试与uv虚拟环境流程。
- 补充流程设计、流程图及Windows平台部署文档。
