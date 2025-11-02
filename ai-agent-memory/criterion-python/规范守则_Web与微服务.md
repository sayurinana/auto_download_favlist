# 规范守则 - Web与微服务

## 项目类型
- REST/GraphQL API服务、后台管理界面、实时推送服务、基于Celery的任务型服务
- 部署形态：容器编排（Kubernetes）、Serverless、PaaS、边缘函数

## 核心原则
### 1. 架构与可扩展性
- 遵循[Twelve-Factor App](https://12factor.net/)原则构建无状态服务，配置与密钥全部外置。
- 按分层/六边形架构组织代码：入口层（Router）→服务层→仓储层；避免业务逻辑出现在框架钩子中。
- 使用Pydantic/FastAPI的`Depends`或Django的`settings`+`settings_components`管理依赖注入。
- 对长耗时请求采用任务队列（Celery / Dramatiq）或异步后台任务，保持API响应时间稳定。

### 2. 可靠性与可观测性
- 全量接入[OpenTelemetry Python SDK](https://opentelemetry.io/docs/languages/python/instrumentation/)收集trace、metrics、logs，统一导出到APM平台。
- 设定SLO：如P95延迟、错误率、吞吐，并在CI/CD中配置门禁（例如Load Test + Prometheus告警）。
- 对外暴露健康检查、就绪检查及指标端点（如`/healthz`、`/readyz`、`/metrics`）。
- 加固安全基线：启用HTTPS、HSTS、CSRF防护、CORS白名单，统一管理API密钥轮换与审计日志。
### 3. 数据与状态管理
- 采用数据库迁移工具（Alembic、Django Migrations），在CI中执行`--check`以防遗漏。
- 使用`pydantic`或`dataclasses`定义DTO/Schema，保证入参出参一致性。
- 缓存策略分层：本地（functools.lru_cache）→分布式（Redis/Memcached），并设计缓存失效策略。
- 对外部服务设置电路熔断器与限流（如`resilience`、`aiobreaker`）。

## 项目骨架示例（FastAPI）
```
app/
├── main.py            # ASGI入口，包含FastAPI实例
├── api/
│   ├── routers.py     # 路由聚合
│   └── deps.py        # 依赖注入
├── core/
│   ├── config.py      # 设置与环境变量
│   ├── logging.py     # 结构化日志
│   └── observability.py
├── services/
├── repositories/
├── workers/           # Celery / RQ
└── tests/
    ├── test_api.py
    └── test_workers.py
```
## 测试与交付
- 单元测试：对服务层、仓储层使用`pytest`+`pytest-asyncio`，隔离外部IO。
- 集成测试：使用`httpx.AsyncClient`或`pytest-django`的`client`模拟HTTP请求，覆盖权限、幂等、限流分支。
- 性能基准：通过`locust`或`k6`建立高并发场景，关注P95延迟与资源使用。
- 安全测试：执行OWASP ZAP、Bandit、pip-audit，验证依赖与常见安全漏洞。
- 部署前参考[Django Deployment Checklist](https://docs.djangoproject.com/en/5.2/howto/deployment/checklist/)逐项核对设置与安全加固。

## 部署与运维
- 使用Docker容器化，镜像遵循最小基镜像、分阶段构建与只读文件系统原则。
- 配置滚动发布与灰度策略，关键接口需配合金丝雀指标监控。
- 结合[FastAPI production best practices](https://github.com/zhanymkanov/fastapi-best-practices)优化Uvicorn/Gunicorn工作进程与超时时间。
- 在AWS等云环境遵循[Microservices on AWS](https://docs.aws.amazon.com/whitepapers/latest/microservices-on-aws/microservices-on-aws.html)白皮书的部署、网络隔离与监控建议。

## 参考资料
- [The Twelve-Factor App](https://12factor.net/) - 云原生应用设计基线。
- [Django Deployment Checklist](https://docs.djangoproject.com/en/5.2/howto/deployment/checklist/) - 官方部署前检查清单。
- [FastAPI Best Practices](https://github.com/zhanymkanov/fastapi-best-practices) - FastAPI生产级落地经验与配置建议。
- [Microservices on AWS](https://docs.aws.amazon.com/whitepapers/latest/microservices-on-aws/microservices-on-aws.html) - 微服务架构运维、监控、安全策略。
- [OpenTelemetry Python Instrumentation](https://opentelemetry.io/docs/languages/python/instrumentation/) - 统一追踪与度量方案。
