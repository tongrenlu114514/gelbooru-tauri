# Phase 4: Polish & Release - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-17
**Phase:** 04-polish-release
**Areas discussed:** Schema Versioning, Error Type, Logging Strategy, Documentation Scope, Release Configuration

---

## Schema Versioning Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| 版本表 + 顺序迁移 | 添加 schema_version 表，new() 按版本顺序执行迁移 SQL | ✓ |
| 仅版本号，无迁移 | 只加版本表记录当前版本，不自动迁移 | |
| 完整迁移框架 | 引入 sqlx migrate 等库，支持 up/down 脚本 | |

**User's choice:** 版本表 + 顺序迁移
**Notes:** 简单可靠，5 张表最多 2-3 个迁移文件即可满足 Phase 4 需求

---

## Error Type Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| 保持 Result<T, String> | 当前模式不变，最小改动，与前端契约不变 | ✓ |
| 引入 thiserror | 结构化错误，区分错误来源 | |
| 引入 anyhow::Error | 命令层用 anyhow::Error 替代 String | |

**User's choice:** 保持 Result<T, String>
**Notes:** Phase 2 CONTEXT 中"anyhow"的描述为过时参考，以实际代码模式为准

---

## Logging Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| 保留 println! | 当前 println! 模式不变，Phase 4 减少风险优先 | ✓ |
| 引入 tracing | 用 tracing::info!/error! 替代 println!，支持日志级别 | |

**User's choice:** 保留 println!()
**Notes:** 发布准备阶段最小改动优先

---

## Documentation Scope

| Option | Description | Selected |
|--------|-------------|----------|
| 基础 README | 功能介绍 + 安装步骤 + 基本使用 + 贡献指南（1-2 页） | ✓ |
| 完整文档 | 包含截图、配置说明、常见问题、架构说明 | |

**User's choice:** 基础 README
**Notes:** 适合快速启动，不包含截图等重量级内容

---

## Release Configuration

| Option | Description | Selected |
|--------|-------------|----------|
| 无需改动 | 版本号已是 1.0.0，无需配置变更，直接构建 | ✓ |
| 添加构建目标 | 配置 NSIS 安装包、MSI 或 portable exe 等发布目标 | |
| 更新 identifier | identifier 是否需要改为正式包名 | |

**User's choice:** 无需改动
**Notes:** tauri.conf.json 版本号已是 1.0.0，identifier 无需变更

---

## Claude's Discretion

- Schema 迁移的具体 SQL 语句由 planner 在实现阶段决定
- README 具体文案由 planner 根据现有功能编写

## Deferred Ideas

- 引入 anyhow/thiserror 统一错误类型 — Phase 4 选择保持现状
- 引入 tracing 替代 println! — Phase 4 选择保持现状
- 完整迁移框架（sqlx migrate / rusqlite_migration 库）— Phase 4 选择手动顺序迁移
- 完整文档（截图、FAQ、架构说明）— Phase 4 选择基础 README
