# 待定项结论

## 🔍 待定项U001 - 程序命名与拼写

### ✅ 方案S001 - 【建议采纳 +80分】
**📍 原文位置**：（ai-agent-memory/now-task.md，第7-9行）
> - 使用rust实现两个程序
>   - 其中auto_download_favlist改名为get_bilibili_favlist_bvid_list
> - 先实现get_bilibili_favlist_bvid_list再实现billibili_favlist_download_helper

**🎯 优化方案**：
> 将新程序统一命名为`bilibili_favlist_download_helper`（修正双l拼写错误），并在Rust工作空间中分别创建`get_bilibili_favlist_bvid_list`与`bilibili_favlist_download_helper`两个二进制crate，保持与历史文档一致。

**⚖️ 权衡分析**：
- ✅ **优势**（+60分）：修正拼写有助于后续查找与文档一致。
- ✅ **优势**（+20分）：统一命名有利于在Cargo工作空间中引用。
- ❌ **风险**（-10分）：与原描述不完全一致需在文档中注明。

---

## 🔍 待定项U002 - Rust 项目结构设计

### ✅ 方案S002 - 【建议采纳 +75分】
**📍 原文位置**：（ai-agent-memory/now-task.md，第5-10行）
> - 使用rust实现两个程序
> ...
> - 新的billibili_favlist_download_helper使用crate的形式调用get_bilibili_favlist_bvid_list的功能

**🎯 优化方案**：
> 建立Cargo工作空间：
> 1. `crates/favlist_core`：封装收藏夹抓取、CSV写入与公共模型，供两个程序共享。
> 2. `crates/get_bilibili_favlist_bvid_list`：可执行二进制，复刻Python版本导出逻辑。
> 3. `crates/bilibili_favlist_download_helper`：交互式二进制，直接依赖`favlist_core`，避免通过Shell调用。

**⚖️ 权衡分析**：
- ✅ **优势**（+50分）：共享核心逻辑，减少重复代码。
- ✅ **优势**（+25分）：满足 “通过crate调用” 的约束，方便未来扩展。
- ❌ **风险**（-15分）：需要维护多crate与工作空间配置，构建复杂度略增。

---

## 🔍 待定项U003 - 收藏夹导出输出格式

### ✅ 方案S003 - 【建议采纳 +70分】
**📍 原文位置**：（py-src/auto_download_favlist/cli.py，第18-86行）
> Python版本导出收藏夹条目为CSV，支持增量去重与自定义编码。

**🎯 优化方案**：
> Rust版本延续CSV输出并默认UTF-8，保留读取既有CSV、去重写入与自定义编码（UTF-8/GBK等），输出结构与历史一致，保证与现有数据、文档兼容。

**⚖️ 权衡分析**：
- ✅ **优势**（+40分）：沿用成熟方案，降低迁移风险。
- ✅ **优势**（+30分）：兼容既有CSV与辅助工具。
- ❌ **风险**（-10分）：实现编码切换需要额外依赖（如`encoding_rs`），测试覆盖需加强。

---

## 🔍 待定项U004 - 交互式菜单输入方案

### ✅ 方案S004 - 【建议采纳 +85分】
**📍 原文位置**：（ai-agent-memory/now-task.md，第9-11行）
> - 只允许用户使用*方向键或WASD，回车键或空格，选中退出项或按下Esc*，来操作功能选项，而不是通过简述功能选项序号来选择

**🎯 优化方案**：
> 使用`crossterm`捕获键盘事件自行渲染菜单，支持↑↓←→与WASD移动选项，Enter/Space确认，Esc返回或退出；通过`ratatui`或轻量自绘确保不会触发数字输入选项。

**⚖️ 权衡分析**：
- ✅ **优势**（+55分）：完全控制输入逻辑，严格满足交互限制。
- ✅ **优势**（+30分）：后续可扩展动画与提示。
- ❌ **风险**（-20分）：手写事件循环与渲染复杂度高，需要额外测试。

---

## 🔍 待定项U005 - 网络与异步栈选择

### ✅ 方案S005 - 【建议采纳 +65分】
**📍 原文位置**：（py-src/auto_download_favlist/fav_client.py，第1-150行）
> Python版本使用`requests`同步请求并分页抓取收藏夹数据。

**🎯 优化方案**：
> Rust版本采用`reqwest` + `tokio` 异步HTTP客户端，请求B站接口、处理超时与分页，输出统一错误类型供两个程序复用。

**⚖️ 权衡分析**：
- ✅ **优势**（+35分）：`reqwest`生态成熟，支持Cookie、超时等能力。
- ✅ **优势**（+30分）：异步实现方便未来并发扩展。
- ❌ **风险**（-15分）：引入异步运行时，二进制入口需设置`tokio::main`，增加实现复杂度。

---

> 注：根据用户指示，本次任务所有待定项由代理自行裁决，无需额外确认。
