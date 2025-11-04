## 🔍 待定项U001 - 任务状态检测与完成判定

### ✅ 方案S001 - 【建议采纳 +70分】
**📍 原文位置**：（ai-agent-memory/now-task.md，第7-8行）
> 添加任务后，对于当前任务状态的检测有问题，经常任务还在运行就输出“下载任务已全部完成”

**🎯 优化方案**：
> 记录每次提交的 BV 号列表，轮询 `/get-tasks/running` 与 `/get-tasks/finished`，仅当全部目标 BV 已消失于运行列表且（若可查询 Aid）出现在完成列表时才结束；若 API 多次异常则延迟重试并提示用户。

**⚖️ 权衡分析**：
- ✅ **优势**（+55分）：精准判定完成状态，避免误报。
- ✅ **优势**（+20分）：可在等待期间展示仍在运行的视频标题。
- ❌ **风险**（-5分）：若服务器不返回 Aid，需要以 Url 模糊匹配，需额外兼容逻辑。

---

## 🔍 待定项U002 - 抓取收藏夹的进度展示

### ✅ 方案S001 - 【建议采纳 +65分】
**📍 原文位置**：（ai-agent-memory/now-task.md，第9-11行）
> 在抓取收藏夹时看不到进度...

**🎯 优化方案**：
> 在 `bilibili_favlist_download_helper` 的导出路径也挂载 `favlist_core` 进度回调，使用 `indicatif::ProgressBar` 显示“已获取/总视频数”；当总数未知时切换为 spinner；抓取完成后总结处理条目和耗时。

**⚖️ 权衡分析**：
- ✅ **优势**（+45分）：长列表抓取过程可视化。
- ✅ **优势**（+20分）：统一 CLI/助手体验。
- ❌ **风险**（-0分）：终端不支持 ANSI 时需降级文本提示。

---

## 🔍 待定项U003 - Windows 独立窗口启动 `bbdown serve`

### ✅ 方案S001 - 【建议采纳 +60分】
**📍 原文位置**：（ai-agent-memory/now-task.md，第12-13行）
> 希望改成不是使用子进程启动，而是在windows中，以独立窗口启动

**🎯 优化方案**：
> 对 `bbdown_auto_launch` 启动逻辑追加平台分支：在 WSL/Windows 环境使用 `powershell.exe -Command Start-Process bbdown -ArgumentList 'serve ...' -WindowStyle Normal`；Linux 原生仍使用子进程。若命令失败则提示手动执行。

**⚖️ 权衡分析**：
- ✅ **优势**（+50分）：用户可直接观察 serve 输出窗口和错误。
- ❌ **风险**（-10分）：WSL→Windows 路径/权限需谨慎处理，需明确 PowerShell 可执行路径。

---

## 🔍 待定项U004 - API 下载目录与本地检测目录分离

### ✅ 方案S001 - 【建议采纳 +75分】
**📍 原文位置**：（ai-agent-memory/now-task.md，第15-21行）
> 把要传给API的存储路径和需要程序检查的存储路径分离开

**🎯 优化方案**：
> 在 `FavConfig` 中新增 `api_download_dir` 与 `scan_download_dir` 字段；请求 BBDown API 时使用 `api_download_dir` 拼接 FilePattern，执行缺漏扫描时使用 `scan_download_dir`。录入与编辑配置时分别提示 Windows 路径与 WSL 路径。

**⚖️ 权衡分析**：
- ✅ **优势**（+60分）：适配跨平台部署（Windows 下载 + WSL 检查）。
- ✅ **优势**（+20分）：便于未来扩展多机下载。
- ❌ **风险**（-5分）：用户需维护两套路径，需在 UI 中清晰说明。

---

## 🔍 待定项U005 - 全局默认配置支持

### ✅ 方案S001 - 【建议采纳 +70分】
**📍 原文位置**：（ai-agent-memory/now-task.md，第23-30行）
> 能在程序的全局配置中设置默认的下载目录/serve地址/File Pattern/Multi File Pattern

**🎯 优化方案**：
> 新增 `global_config.json`（或 `defaults.json`）存放全局默认项；程序启动时先读取全局配置，再在录入新收藏夹或编辑时用作默认值显示，若用户跳过则写入配置。提供 CLI 选项用于更新全局默认。

**⚖️ 权衡分析**：
- ✅ **优势**（+55分）：多收藏夹场景减少重复输入。
- ✅ **优势**（+20分）：可统一调整 serve 地址或文件模板。
- ❌ **风险**（-5分）：需设计兼容老版本（无全局配置）时的回退逻辑。

---
