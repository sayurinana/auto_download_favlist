# AI_AGENT.md - CLI应用规范守则

这是针对Rust命令行工具开发的通用AI代理规范守则文档。

## 项目类型
命令行应用程序(CLI Tool)

## 核心要求

### 用户体验
- 使用`clap`进行参数解析，支持子命令和选项
- 提供详细的帮助信息和使用示例
- 错误信息友好，包含建议解决方案
- 支持彩色输出（使用`colored`或`console`）
- 长时间操作显示进度条（使用`indicatif`）

### 配置管理
- 支持配置文件（TOML/YAML格式）
- 支持环境变量覆盖
- 遵循XDG Base Directory规范（使用`dirs`库）
- 配置验证和默认值处理

### 错误处理
- 使用`anyhow`进行错误传播
- 区分用户错误和系统错误
- 提供合适的退出码
- 错误信息包含上下文和建议

### 文件操作
- 使用`std::path::Path`而非字符串拼接
- 支持相对路径和绝对路径
- 处理权限错误和不存在文件
- 备份重要文件操作

## 必需依赖
```toml
[dependencies]
clap = { version = "4.0", features = ["derive"] }
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
dirs = "5.0"
```

## 推荐依赖
```toml
# 用户界面
colored = "2.0"
indicatif = "0.17"
console = "0.15"

# 配置文件
toml = "0.8"
serde_yaml = "0.9"

# 日志记录
env_logger = "0.10"
log = "0.4"
```

## 项目结构
```
src/
├── main.rs          # 入口点，参数解析
├── cli.rs           # 命令行接口定义
├── config.rs        # 配置文件处理
├── commands/        # 子命令实现
│   ├── mod.rs
│   ├── process.rs
│   └── validate.rs
├── error.rs         # 错误类型定义
└── utils.rs         # 工具函数
```

## 代码模板

### CLI定义
```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mycli")]
#[command(about = "工具描述")]
pub struct Cli {
    /// 配置文件路径
    #[arg(short, long)]
    pub config: Option<PathBuf>,
    
    /// 详细输出
    #[arg(short, long)]
    pub verbose: bool,
    
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 处理文件
    Process {
        /// 输入文件
        input: PathBuf,
        /// 输出文件
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}
```

### 主函数结构
```rust
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    // 设置日志级别
    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug");
    }
    env_logger::init();
    
    match cli.command {
        Commands::Process { input, output } => {
            commands::process::run(&input, output.as_ref())?;
        }
    }
    
    Ok(())
}
```

## 测试要求
- 使用`assert_cmd`进行CLI集成测试
- 测试各种参数组合和错误情况
- 使用临时目录进行文件操作测试
- 验证帮助信息和退出码

## 发布准备
- 支持多平台编译（Windows/macOS/Linux）
- 提供安装脚本或包管理器集成
- 文档包含安装和使用说明
- 考虑提供shell补全脚本
