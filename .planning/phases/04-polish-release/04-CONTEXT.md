# Phase 4: Polish & Release - Context

**Gathered:** 2026-04-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Final polish and release preparation — database schema versioning, error handling consistency, documentation update, and release configuration verification.

</domain>

<decisions>
## Implementation Decisions

### 4.1 数据库 Schema 版本管理

- **D-01:** 策略 = 版本表 + 顺序迁移
  - 在 `db/mod.rs` 中添加 `schema_version` 表（id, version INTEGER）
  - `new()` 初始化时读取当前版本，按顺序执行迁移 SQL
  - 每个迁移是独立的 `conn.execute()` 调用，按版本号顺序执行
  - 当前表已通过 `CREATE TABLE IF NOT EXISTS` 创建，初始版本设为 1

- **D-02:** 迁移文件名规范
  - `001_init.sql`（现有 5 张表的初始版本）
  - 未来迁移：`002_add_column.sql` 等，按序号顺序执行

### 4.2 错误处理统一化

- **D-03:** 错误类型 = 保持 `Result<T, String>`
  - 命令层继续返回 `Result<T, String>`
  - `.map_err(|e| format!("..."))` 模式不变
  - Phase 2 CONTEXT 中"anyhow"的描述为过时参考，以实际代码为准

- **D-04:** 日志策略 = 保留 `println!()`
  - 当前散落在 gelbooru.rs、download.rs 等文件的 `println!("[DEBUG]...")` / `println!("[ERROR]...")` 模式不变
  - Phase 4 不引入 tracing 或 log 框架
  - 理由：发布准备阶段最小改动优先

### 4.3 文档完善

- **D-05:** README 范围 = 基础 README
  - 功能介绍 + 安装步骤 + 基本使用 + 贡献指南
  - 1-2 页，适合快速启动
  - 不包含截图、架构说明、FAQ（留待未来扩展）

### 4.4 发布配置

- **D-06:** tauri.conf.json 无需改动
  - 版本号已是 `1.0.0`
  - identifier 和 productName 无需变更
  - 构建目标使用 Tauri 默认配置

### Claude's Discretion

- Schema 迁移的具体 SQL 语句由 planner 在实现阶段决定
- README 具体文案由 planner 根据现有功能编写

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Context
- `.planning/phases/02-quality-testing/02-CONTEXT.md` — Phase 2 decisions (coverage, Rust test patterns)
- `.planning/phases/03-performance-reliability/03-CONTEXT.md` — Phase 3 decisions (download retry, rate limit)
- `.planning/ROADMAP.md` § Phase 4 — Phase 4 goal and task list

### Project Docs
- `.planning/PROJECT.md` — Tech stack: Tauri 2.x, Vue 3, rusqlite, reqwest
- `.planning/STATE.md` — Current state: Phase 3 complete, Phase 4 pending

### Codebase Patterns
- `src-tauri/src/db/mod.rs` — Database implementation with `CREATE TABLE IF NOT EXISTS` (no versioning yet)
- `src-tauri/src/commands/gelbooru.rs` — `Result<T, String>` + `println!()` pattern examples
- `src-tauri/src/commands/download.rs` — Download command error handling pattern
- `src-tauri/tauri.conf.json` — Already at version 1.0.0

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `Database` struct (`db/mod.rs`): 已有连接管理基础，添加 schema_version 表只需扩展 `new()` 逻辑
- `SqliteResult` 别名: `rusqlite::Result as SqliteResult` — 统一使用

### Established Patterns
- `Result<T, String>` 命令返回值风格：前端 `invoke` 直接 `.catch()` 处理
- `println!()` 日志：按 `[DEBUG]` / `[ERROR]` 前缀区分，日志输出到 stderr
- `CREATE TABLE IF NOT EXISTS`：所有现有表均使用此模式

### Integration Points
- Schema 迁移：在 `Database::new()` 中，`Connection::open()` 之后立即执行版本检查和迁移
- README 更新：`README.md` 位于项目根目录

</code_context>

<specifics>
## Specific Ideas

No specific requirements beyond decisions above — open to standard approaches for README content.

</specifics>

<deferred>
## Deferred Ideas

- 引入 anyhow/thiserror 统一错误类型 — Phase 4 选择保持现状，未来有需求再提
- 引入 tracing 替代 println! — Phase 4 选择保持现状，未来有需求再提
- 完整迁移框架（sqlx migrate / rusqlite_migration 库）— Phase 4 选择手动顺序迁移，不过度工程化
- 完整文档（截图、FAQ、架构说明）— Phase 4 选择基础 README

---

*Phase: 04-polish-release*
*Context gathered: 2026-04-17*
