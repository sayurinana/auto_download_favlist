# 流程设计 - B站收藏夹导出工具

## 一、总体结构
- `src/auto_download_favlist/cli.py`：Typer命令入口，解析参数并调度抓取/持久化流程。
- `src/auto_download_favlist/fav_client.py`：封装B站收藏夹API调用，负责分页、错误处理与收藏夹元数据获取。
- `src/auto_download_favlist/models.py`：使用`dataclasses`定义收藏夹与视频条目数据结构，统一字段命名。
- `src/auto_download_favlist/csv_writer.py`：负责去重、编码处理与CSV写入。
- `src/auto_download_favlist/utils.py`：通用工具（URL解析、时间戳生成、日志配置）。
- `tests/`：包含API响应mock、CLI端到端测试、CSV写入去重测试。

## 二、执行步骤与依赖关系
1. **准备阶段**
   - 初始化`src/`与`tests/`目录，配置`pyproject.toml`或`requirements.txt`声明`typer`、`requests`、`pydantic`（若需）等依赖。
   - 建立`__init__.py`与基础日志工具。
2. **收藏夹API实现**
   - 提取URL中的`fid`/`media_id`参数，构造`https://api.bilibili.com/x/v3/fav/folder/info`获取收藏夹名称与数量。
   - 使用`https://api.bilibili.com/x/v3/fav/resource/list?media_id=<id>&pn=<page>&ps=<size>`分页拉取视频条目。
   - 捕获HTTP异常与B站业务错误码，按需重试或提示用户提供Cookie。
3. **数据模型与转换**
   - 将API返回字段映射为`VideoEntry`（BV号、标题、收藏夹名、时间戳）。
   - 支持通过CLI参数指定时间戳格式（默认`%Y-%m-%dT%H-%M-%S`）。
4. **CSV去重写入**
   - 若输出文件存在，先读取已有BV号集合。
   - 合并新数据后排序（按BV或抓取时间），用指定编码写入。
   - 追加模式写入新条目并输出写入数量。
5. **CLI对接与输出**
   - 定义命令：`python -m auto_download_favlist.cli <fav_url> --output favlist.csv --encoding gbk --cookie "SESSDATA=..."`。
   - 捕获异常统一输出友好提示并返回非零退出码。
6. **流程图生成**
   - 在`program_flowchart/src/fav_export.txt`编写PlantUML，生成PNG存放于`program_flowchart/png/`，并复制到任务输出目录。

## 三、测试计划
- **单元测试**：
  - `tests/test_fav_client.py`：模拟API响应验证分页与错误处理。
  - `tests/test_csv_writer.py`：覆盖去重、编码参数、文件存在/不存在分支。
- **集成测试**：
  - 使用`pytest`调用CLI并mock API，验证参数组合与输出文件内容。
- **手动验证**：
  - 对示例收藏夹执行真实请求，检查条目数量、CSV编码，并记录结果。

## 四、风险与降级方案
- **API需要Cookie/签名**：允许用户通过参数传入Cookie；若仍失败，提示可能权限受限并输出错误码。
- **分页条目超量**：提供`--page-size`参数以便调优；默认40与前端一致。
- **请求限流**：对API调用设置重试+指数退避（3次），超限时记录日志并提前退出。
- **Windows兼容性**：避免使用`Pathlib`中不兼容特性，编码转换使用`codecs`确保GBK写入。

## 五、工具与交付要求
- 依赖：`python>=3.10`、`typer`、`requests`、`rich`（用于彩色日志，可选）、`pytest`、`pytest-mock`。
- 生成物：流程图PNG、PlantUML源文件、脚本源代码、测试、Windows部署指南、更新后的`CHANGELOG.md`。
- 所有阶段完成后更新`AI-AGENT_working-status.csv`并根据规范提交。
