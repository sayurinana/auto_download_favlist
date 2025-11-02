# Windows 部署与运行指引

## 1. 环境要求
- Windows 10 及以上版本，具备PowerShell 5或Windows Terminal。
- 已安装 [Python 3.11+](https://www.python.org/downloads/windows/) 并勾选“Add Python to PATH”。
- 已安装 [uv](https://docs.astral.sh/uv/getting-started/installation/)（推荐使用`pip install uv`或`scoop install uv`）。
- 可访问互联网以调用B站开放接口，必要时准备有效的账号Cookie。

## 2. 获取代码
```powershell
# 建议在PowerShell中执行
cd C:\workspace
git clone https://example.com/auto_download_favlist.git
cd auto_download_favlist
```

## 3. 创建与激活虚拟环境
```powershell
uv venv
# PowerShell 激活
.\.venv\Scripts\Activate.ps1
```
若使用CMD：`\.venv\Scripts\activate.bat`。

## 4. 安装依赖
```powershell
uv pip install -r requirements.txt
```
> 依赖中锁定`click>=8.1,<8.2`以保持与Typer兼容；`uv`会自动解决余下依赖。

## 5. 运行示例
```powershell
$env:PYTHONPATH = "src"
uv run python -m auto_download_favlist.cli "https://space.bilibili.com/234561771/favlist?fid=3670113371" `
    --output .\output\favlist.csv `
    --encoding gbk
```
- 如需传入Cookie：加上`--cookie "SESSDATA=xxxx; bili_jct=yyyy"`。
- `--output`可指向自定义路径，默认编码GBK，亦可通过`--encoding utf-8`指定。

## 6. 常见问题
- **HTTP 412或412错误**：未设置`User-Agent`/`Referer`或Cookie失效，确认账号权限。
- **编码写入失败**：使用`--encoding utf-8`并确认目标文件支持该编码。
- **重复数据**：脚本会自动读取既有CSV并去重，如需刷新可删除旧文件后重跑。

## 7. 后续维护建议
- 定期更新依赖：`uv pip install --upgrade -r requirements.txt`。
- 结合Windows任务计划程序，可编写`.ps1`脚本周期性执行抓取。
- 若收藏夹为私密，需要登录后的Cookie方可访问，注意不要将Cookie写入公共仓库。
