# AI_AGENT.md - Web服务规范守则

这是针对Rust Web应用和API服务开发的通用AI代理规范守则文档。

## 项目类型
Web服务/API服务

## 核心要求

### 架构设计
- 使用分层架构：Handler -> Service -> Repository
- 依赖注入和配置管理
- 中间件系统设计
- 优雅关闭和信号处理

### 性能和并发
- 基于tokio的异步架构
- 连接池管理（数据库、Redis等）
- 请求限流和超时处理
- 响应缓存策略

### 安全性
- 输入验证和清理
- 认证和授权机制
- CORS和安全头设置
- 敏感信息保护

### 可观测性
- 结构化日志记录
- 指标收集和监控
- 分布式链路追踪
- 健康检查端点

## 推荐Web框架
```toml
# 选择其一作为主框架
axum = "0.7"           # 推荐，类型安全
# actix-web = "4.0"    # 高性能选择
# warp = "0.3"         # 函数式风格
```

## 必需依赖
```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
thiserror = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
```

## 推荐依赖
```toml
# 数据库
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "chrono", "uuid"] }

# 配置管理
config = "0.13"
dotenvy = "0.15"

# 日志和监控
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# 验证
validator = { version = "0.16", features = ["derive"] }

# 安全
bcrypt = "0.15"
jsonwebtoken = "9.0"
secrecy = "0.8"
```

## 项目结构
```
src/
├── main.rs              # 应用入口
├── lib.rs               # 库定义
├── config/              # 配置管理
│   ├── mod.rs
│   └── database.rs
├── handlers/            # HTTP处理器
│   ├── mod.rs
│   ├── auth.rs
│   └── users.rs
├── services/            # 业务逻辑服务
│   ├── mod.rs
│   ├── auth_service.rs
│   └── user_service.rs
├── repositories/        # 数据访问层
│   ├── mod.rs
│   └── user_repository.rs
├── models/              # 数据模型
│   ├── mod.rs
│   ├── user.rs
│   └── auth.rs
├── middleware/          # 中间件
│   ├── mod.rs
│   ├── auth.rs
│   └── logging.rs
├── errors.rs            # 错误处理
└── utils.rs             # 工具函数
```

## 代码模板

### 应用启动
```rust
use anyhow::Result;
use axum::{Router, serve};
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化追踪
    tracing_subscriber::fmt::init();
    
    // 加载配置
    let config = Config::from_env()?;
    
    // 初始化数据库
    let db_pool = create_db_pool(&config.database_url).await?;
    
    // 构建应用状态
    let app_state = AppState {
        db: db_pool,
        config: config.clone(),
    };
    
    // 创建路由
    let app = Router::new()
        .nest("/api/v1", api_routes())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);
    
    // 启动服务器
    let listener = TcpListener::bind(&config.bind_address).await?;
    tracing::info!("服务启动于 {}", config.bind_address);
    
    serve(listener, app).await?;
    Ok(())
}
```

### 错误处理
```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("数据库错误")]
    Database(#[from] sqlx::Error),
    
    #[error("验证失败: {0}")]
    Validation(String),
    
    #[error("未授权")]
    Unauthorized,
    
    #[error("资源未找到")]
    NotFound,
    
    #[error("内部服务器错误")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(_) => {
                tracing::error!("数据库错误: {}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "内部服务器错误")
            }
            AppError::Validation(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "未授权"),
            AppError::NotFound => (StatusCode::NOT_FOUND, "资源未找到"),
            AppError::Internal(_) => {
                tracing::error!("内部错误: {}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "内部服务器错误")
            }
        };

        let body = Json(json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}
```

### 处理器示例
```rust
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    
    #[validate(email)]
    pub email: String,
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<User>, AppError> {
    // 验证输入
    payload.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    
    // 调用服务层
    let user = state.user_service
        .create_user(payload.name, payload.email)
        .await?;
    
    Ok(Json(user))
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<User>, AppError> {
    let user = state.user_service
        .get_user(user_id)
        .await?
        .ok_or(AppError::NotFound)?;
    
    Ok(Json(user))
}
```

### 服务层
```rust
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserService {
    db: PgPool,
}

impl UserService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
    
    pub async fn create_user(
        &self,
        name: String,
        email: String,
    ) -> Result<User, AppError> {
        // 检查邮箱是否已存在
        if self.email_exists(&email).await? {
            return Err(AppError::Validation("邮箱已存在".to_string()));
        }
        
        // 创建用户
        let user_id = Uuid::new_v4();
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (id, name, email, created_at)
            VALUES ($1, $2, $3, NOW())
            RETURNING id, name, email, created_at
            "#,
            user_id,
            name,
            email
        )
        .fetch_one(&self.db)
        .await?;
        
        tracing::info!("用户创建成功: {}", user.id);
        Ok(user)
    }
    
    async fn email_exists(&self, email: &str) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM users WHERE email = $1",
            email
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count > 0)
    }
}
```

## 配置管理
```rust
use config::{Config, ConfigError};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub bind_address: String,
    pub database_url: String,
    pub jwt_secret: String,
    pub log_level: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(config::Environment::with_prefix("APP"))
            .add_source(config::File::with_name("config").required(false))
            .set_default("bind_address", "0.0.0.0:3000")?
            .set_default("log_level", "info")?
            .build()?
            .try_deserialize()
    }
}
```

## 测试要求
- 使用`tokio-test`进行异步测试
- 集成测试使用测试数据库
- HTTP处理器单元测试
- 使用`httptest`进行端到端测试

## 部署配置
- 支持Docker容器化部署
- 健康检查端点实现
- 优雅关闭信号处理
- 环境变量配置管理
- 日志JSON格式输出
