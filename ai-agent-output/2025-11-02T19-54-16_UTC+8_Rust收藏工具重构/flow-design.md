# 流程设计

## 1. 总体架构
- 创建 Cargo 工作空间，统一管理三个 crate：`favlist_core`（lib）、`get_bilibili_favlist_bvid_list`（bin）、`bilibili_favlist_download_helper`（bin）。
- `favlist_core` 提供：
  - `client` 模块：封装 B 站收藏夹 API 调用、分页迭代、错误类型。
  - `csv` 模块：读取历史 CSV、增量去重、写入新条目、编码转换。
  - `models` 模型与 DTO。
  - `time` 工具：时间戳格式化。
  - `inventory` 工具：目录扫描与缺漏比对（供 helper 使用）。
  - `bbdown` trait：抽象下载命令执行（helper 实现具体版本）。
- CLI/助手 crate 通过同步接口调用核心库，保证调用简单。内部实现可异步→同步封装。

## 2. 关键流程
1. **收藏夹导出 CLI**
   - 解析命令行 → 验证参数 → 调用 `favlist_core::export::export_to_csv`。
   - `export_to_csv`：构建 HTTP 客户端 → 获取收藏夹信息 → 加载历史 CSV → 遍历分页 → 过滤重复 → 写入新 CSV。
   - 输出总结信息与退出码（成功0、参数错误退出码2、API失败退出1等）。

2. **交互式助手**
   - 启动时加载配置仓库（JSON 文件，XDG config）。
   - 主菜单状态机（`MainMenu`)：录入新收藏夹 / 使用存档 / 退出。
   - 方向键/WASD 更新光标，Enter/Space 确认，Esc 回退。
   - “录入新收藏夹”：收集URL和目录 → 调用导出 → 保存新配置。
   - “使用存档”：选择配置后进入子菜单（编辑 / 检查更新 / 检查缺漏 / 返回）。
     - 编辑：支持修改各字段并写回仓库。
     - 检查更新：备份旧 CSV → 调用导出 → `diff_new_entries` → 调用 bbdown（或dry-run打印）。
     - 检查缺漏：扫描下载目录 → 比对 CSV → 调用 bbdown。

3. **流程图输出**
   - 通过 PlantUML 描述导出流程与助手流程，生成 PNG。

## 3. 数据与文件
- 配置文件路径：默认 `~/.config/bilibili_favlist_helper/config.json`（尊重 `XDG_CONFIG_HOME`）。Windows 使用 `%APPDATA%`。
- CSV 输出：默认放置在配置指定下载目录，命名 `YYYY-MM-DDThh-mm-ss-favlist.csv`。
- 备份命名：`*.backup.csv`。
- 目录清单：写入 `download_dir/inventory_<timestamp>.json` 或 `.txt`（待定，计划使用 JSON）。

## 4. 测试计划
- `favlist_core`：
  - 使用 `httpmock` 模拟 B 站接口，覆盖分页、错误码、空列表。
  - CSV 单元测试：UTF-8/GBK 编码读写、去重逻辑。
  - 目录扫描 & 差异比对测试。
- `get_bilibili_favlist_bvid_list`：参数解析、错误提示与零新增路径单测。
- `bilibili_favlist_download_helper`：
  - 状态机逻辑使用虚拟输入序列测试。
  - bbdown trait 替换为假实现验证 dry-run。
- 集成验证脚本：手动运行示例收藏夹，收集日志。

## 5. 里程碑拆分
1. 初始化 Cargo 工作区 + core crate 骨架。
2. 完成 HTTP 抓取 + CSV 去重，写基础测试。
3. 实现导出 CLI，验证命令行输出。
4. 实现配置仓库与bbdown抽象。
5. 构建交互式菜单、状态机与视图渲染。
6. 增补测试、流程图、文档与CHANGELOG。
7. 自检表核对、收尾。

## 6. 风险与预案
- **HTTP 接口依赖**：若 B 站返回结构变化，核心库需容错字段缺失；编写解析容错逻辑。
- **终端兼容性**：若终端不支持 crossterm 所需特性，提供回退提示（记录到文档）。
- **编码问题**：通过自动检测（BOM、配置）+ 单测验证，必要时在 CLI 提示如何处理失败。
- **bbdown 缺失**：捕获 `std::io::ErrorKind::NotFound`，输出安装指引。
