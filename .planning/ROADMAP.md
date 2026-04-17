# Gelbooru Downloader - Roadmap

**Last Updated:** 2026-04-17

## Project Phases

### Phase 1: Foundation & Polish
**Goal:** Fix critical issues and establish project foundation

| # | Task | Priority | Files |
|---|------|----------|-------|
| 1.1 | 设置持久化到数据库 | HIGH | settings.ts, db module |
| 1.2 | 下载任务状态持久化 | HIGH | download.rs, download store |
| 1.3 | 路径清理和安全验证 | HIGH | gallery.rs |
| 1.4 | 移除硬编码路径 | HIGH | gallery.rs, http.rs, settings.ts |

**Success Criteria:**
- 设置重启后可恢复
- 下载任务重启后可继续
- 无硬编码路径
- 路径操作安全

### Phase 2: Quality & Testing
**Goal:** Add testing and improve code quality

| # | Task | Priority | Files |
|---|------|----------|-------|
| 2.1 | 配置测试框架 (Vitest + Rust tests) | MEDIUM | package.json, Cargo.toml |
| 2.2 | 单元测试 (前端) | MEDIUM | src/**/*.spec.ts |
| 2.3 | 单元测试 (后端) | MEDIUM | src-tauri/src/**/*.rs |
| 2.4 | 配置 ESLint/Prettier | MEDIUM | eslint.config.js, prettier.config.js |

**Success Criteria:**
- 测试框架就绪
- 核心功能有测试覆盖
- 代码风格统一

**Plans:**
5/5 plans executed
- [x] 02-02-PLAN.md - Frontend gallery store page state tests
- [x] 02-03-PLAN.md - Backend database CRUD tests with tempfile
- [x] 02-04-PLAN.md - Backend scraper HTML parsing tests
- [x] 02-05-PLAN.md - Husky pre-commit hook setup

### Phase 3: Performance & Reliability
**Goal:** Improve performance and add reliability features

| # | Task | Priority | Files |
|---|------|----------|-------|
| 3.1 | imageCache 内存泄漏修复 | HIGH | Gallery.vue |
| 3.2 | 下载重试机制 | MEDIUM | download.rs |
| 3.3 | 大目录扫描优化 | MEDIUM | gallery.rs |
| 3.4 | 添加请求限流 | LOW | http.rs |

**Plans:**
- [x] 03-01-PLAN.md - imageCache lazy loading via IntersectionObserver
- [x] 03-02-PLAN.md - Download retry with exponential backoff (3 retries, 1s/2s/4s)
- [x] 03-03-PLAN.md - Async directory scan with tokio::fs + Semaphore (max 10 handles)
- [x] 03-04-PLAN.md - Global HTTP rate limiting (500ms gap via RwLock<Instant>)

**Success Criteria:**
- 无内存泄漏
- 下载失败可重试
- 大目录操作流畅

### Phase 4: Polish & Release
**Goal:** Final polish and release

| # | Task | Priority | Files |
|---|------|----------|-------|
| 4.1 | 数据库 schema 版本管理 | MEDIUM | db/mod.rs |
| 4.2 | 错误处理统一化 | MEDIUM | 全局 |
| 4.3 | 文档完善 | LOW | README.md |
| 4.4 | 发布准备 | HIGH | tauri.conf.json |

**Success Criteria:**
- Schema 版本可追踪
- 错误信息统一友好
- 文档完整
- 可发布版本

## State

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 1 | COMPLETED | 4/4 tasks done |
| Phase 2 | COMPLETED | 5/5 plans done |
| Phase 3 | COMPLETED | 4/4 plans executed |
| Phase 4 | PENDING | 未开始 |

## Recent Commits

```
d07745c feat(tags): 添加标签收藏功能
c702cfc fix(gallery): 修复恢复页面状态时页码被重置为1的问题
ce8fa2b refactor(gallery): 移除搜索结果缓存，仅保留页面状态恢复
235182b feat(gallery): 页面切换后恢复搜索页面状态
342a78c feat(gallery): 缓存结果包含搜索条件
```
