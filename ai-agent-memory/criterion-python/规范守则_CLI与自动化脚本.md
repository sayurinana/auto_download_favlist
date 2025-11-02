# 规范守则 - CLI与自动化脚本

## 项目类型
- 命令行工具、DevOps脚本、批处理任务、桌面自动化脚本
- 主要交付形态：单一命令入口、多子命令工具、计划任务/守护进程

## 核心原则
### 1. 体验与可发现性
- 使用`typer`或`click`提供自动补全、彩色输出与结构化帮助信息。
- 默认输出应贴近人类可读格式；通过`--json`等开关提供机器可读结果。
- 在错误信息中提供修复建议与常见问题链接，失败时返回非零退出码。

### 2. 配置与可移植性
- 读取顺序：命令行参数 > 环境变量 > 配置文件（`pyproject.toml`/YAML）。
- 遵循[XDG Base Directory](https://specifications.freedesktop.org/basedir-spec/latest/)管理用户态配置与缓存。
- 使用`packaging.python.org`推荐的结构打包，可选`poetry`/`hatch`管理依赖与发布。

### 3. 安全与可维护性
- 对外暴露的命令默认启用`--yes`/`--dry-run`等保护开关，关键操作需再次确认。
- 在敏感操作前写入审计日志（字段含命令参数、操作对象、结果码）。
- 定期通过`pip-audit`或`pipenv check`执行依赖安全扫描。
## 项目骨架建议
```
project/
├── pyproject.toml
├── src/
│   └── project/
│       ├── __init__.py
│       ├── cli.py        # Typer/Click命令定义
│       ├── config.py     # 配置加载与验证
│       ├── services/     # 业务逻辑分层
│       └── runners.py    # 执行入口
├── tests/
│   ├── test_cli.py
│   └── fixtures/
└── docs/
    └── usage.md
```

### 示例启动器
```python
import typer
from project import services

app = typer.Typer(help="批量图片处理工具")

@app.command()
def resize(input_dir: typer.Path, width: int = 1280, height: int = 720, dry_run: bool = False):
    services.resize_batch(input_dir, width, height, dry_run=dry_run)

if __name__ == "__main__":
    app()
```
## 测试与交付策略
- 使用`pytest`+`pytest-console-scripts`或`typer.testing.CliRunner`覆盖命令执行路径。
- 在CI中针对不同平台（Windows/macOS/Linux）运行烟雾测试，验证编码与路径兼容性。
- 对自动化脚本建立回归测试用例，使用`freezegun`或`pytest-mock`隔离时间/外部依赖。
- 发布前执行`python -m build`与`twine check`验证包元数据；采用`pipx`进行安装验证。

## 运行与运维
- 为关键命令提供`--log-level`与`--log-file`；使用`structlog`或`logging` JSONFormatter输出机器可读日志。
- 通过`rich`或`tqdm`反馈长任务进度，控制在可配置的刷新频率内。
- 结合`cron`/`systemd`/GitHub Actions调度时，增加幂等设计与失败重试策略。
- 对外部系统调用设置超时、重试与指数退避参数，避免脚本无限挂起。

## 参考资料
- [Creating and packaging command-line tools](https://packaging.python.org/en/latest/guides/creating-command-line-tools/) - Python Packaging Authority官方指南。
- [Typer Documentation](https://typer.tiangolo.com/) - Typer官方文档，涵盖类型注解驱动的CLI设计与测试工具。
- [Click Documentation](https://click.palletsprojects.com/en/stable/) - Click官方文档，提供命令分组、参数校验、自动帮助生成等细节。
- [PEP 8 - Style Guide for Python Code](https://peps.python.org/pep-0008/) - Python官方编码规范，约束脚本代码风格。
- [Google Python Style Guide](https://google.github.io/styleguide/pyguide.html) - 大型团队通用的Python风格与Lint约束，可用作补充参考。
