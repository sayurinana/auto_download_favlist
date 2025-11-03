## 🔍 待定项U001 - 界面美化方案落实

### ✅ 方案S001 - 【建议采纳 +70分】
**📍 原文位置**：（ai-agent-memory/now-task.md，第4-6行）
> 使用`console`或其他合适的crate来美化界面的输出和色彩等界面设计

**🎯 优化方案**：
> 继续沿用`console`作为基础着色库，并补充`console::style`配合`console::Emoji`、`console::Term`封装输出；同时引入`indicatif`用于统一进度展示与菜单提示色彩，保持与核心库共享的风格。

**⚖️ 权衡分析**：
- ✅ **优势**（+60分）：沿用成熟crate，维护难度低且与现有CLI一致。
- ✅ **优势**（+20分）：`indicatif`提供现成多字段进度条，满足“当前/总量”需求。
- ❌ **风险**（-10分）：终端颜色依赖ANSI支持，需兼顾Windows兼容性。

---

## 🔍 待定项U002 - 收藏抓取进度回报机制

### ✅ 方案S001 - 【建议采纳 +75分】
**📍 原文位置**：（ai-agent-memory/now-task.md，第6-8行）
> 在抓取收藏夹时显示进度条（显示当前已获取数和总视频数而不是最大100%的百分比）

**🎯 优化方案**：
> 在`favlist_core::export`内部新增进度回调接口，按页面/条目完成后回报`current`与`total`；CLI通过`indicatif::ProgressBar`展示“已获取/总数”并在未知总量时回退为spinner。进度回调使用`Arc<dyn Fn>`维持跨线程安全。

**⚖️ 权衡分析**：
- ✅ **优势**（+50分）：核心库对外暴露统一进度事件，可被多个工具复用。
- ✅ **优势**（+30分）：用户获取明确数量信息，符合需求描述。
- ❌ **风险**（-5分）：需要更新现有调用方（至少两个crate），需兼容老行为。

---

## 🔍 待定项U003 - BBDown serve 启动与配置策略

### ✅ 方案S001 - 【建议采纳 +80分】
**📍 原文位置**：（ai-agent-memory/now-task.md，第10-20行）
> 检查缺漏时，先在一个子进程中启动`BBDown serve`...此地址可通过配置指定...

**🎯 优化方案**：
> 在助手配置结构中新增`bbdown_serve_url`、`bbdown_auto_launch`、`bbdown_launch_args`、`bbdown_poll_interval_ms`字段，提供默认`http://localhost:23333`与500ms轮询；`--dry-run`或Linux平台缺省时跳过实际启动，仅打印命令。通过`Command`启动时监测可执行文件是否存在，并在程序退出或任务完成后显式终止子进程。

**⚖️ 权衡分析**：
- ✅ **优势**（+60分）：配置可持久化并可覆盖用户自建服务器。
- ✅ **优势**（+25分）：自动关闭子进程，避免残留服务。
- ❌ **风险**（-5分）：Windows/Linux启动参数存在差异，需要补充测试与错误提示。

---

## 🔍 待定项U004 - 缺漏补全的任务调度与Dry-Run策略

### ✅ 方案S001 - 【建议采纳 +70分】
**📍 原文位置**：（ai-agent-memory/now-task.md，第16-24行）
> 对于找到的缺漏项，通过向API发起http请求来添加下载任务...下载完成后...dry-run则无动作

**🎯 优化方案**：
> 封装`bbdown::ApiClient`使用`reqwest::blocking::Client`，提供`add_task`、`list_running`、`list_finished`与`wait_until_idle`方法；缺漏补全阶段批量提交所有BVID，dry-run模式打印拟提交JSON并跳过HTTP请求，再按配置轮询，空闲后触发再次检查但不重复提交。失败请求或超时统一提示并允许手动退出。

**⚖️ 权衡分析**：
- ✅ **优势**（+55分）：API client封装减少主流程复杂度。
- ✅ **优势**（+20分）：Dry-run可在无BBDown环境下验证逻辑。
- ❌ **风险**（-5分）：轮询阻塞UI时间，需配合提示文本缓解等待感。
## 🔍 待定项U005 - FilePattern与多文件模式配置策略

### ✅ 方案S001 - 【建议采纳 +80分】
**📍 原文位置**：（ai-agent-memory/now-task.md，第26-31行）
> 需要在配置中支持单独设置保存file-pattern和multi-file-pattern，仅在调用API时按配置拼接路径与模式。

**🎯 优化方案**：
> 扩展`FavConfig`新增`file_pattern`与`multi_file_pattern`字段，并保留原有下载目录；在调用BBDown API时，将下载目录路径与所选Pattern拼接为`FilePattern`字段（转义反斜杠），若存在multi-pattern则提供给`MultiFilePattern`；否则沿用bbdown.config默认配置。配置编辑界面允许用户维护两个新字段，对应dry-run时仅打印最终拼接结果。

**⚖️ 权衡分析**：
- ✅ **优势**（+60分）：满足API模式字段需求，与现有配置持久化结合良好。
- ✅ **优势**（+25分）：保持与bbdown.config共存，只在API调用路径上动态覆盖。
- ❌ **风险**（-5分）：Windows/Linux路径差异需注意转义处理，需补充测试。
