---
phase: 3
phase_name: Performance & Reliability
status: discussed
last_updated: "2026-04-16"
---

# Phase 3: Performance & Reliability - Context

## Phase Boundary

Improve application performance and reliability — fix memory leaks, add retry mechanisms, optimize large directory scanning, and add request rate limiting.

## Implementation Decisions

### 3.1 imageCache 内存泄漏修复

- **D-01:** 内存泄漏根源 = `preloadImages` 预加载失控
  - 当前 `preloadImages` 无限制遍历所有图片路径，全部转为 base64 入缓存
  - 图片数量多时 base64 字符串内存暴涨

- **D-02:** 修复方案 = 懒加载 + 缓存
  - 只加载屏幕内可见图片
  - 屏幕外图片从缓存移除
  - 保留 `convertFileSrc` 直连路径（非 base64，不占缓存）
  - LRU 缓存继续管理 base64 路径的内存上限

- **D-03:** 具体改动位置 = `src/views/Gallery.vue`
  - 修改 `preloadImages` → `loadVisibleImages`
  - 利用 IntersectionObserver 触发懒加载
  - 屏幕外图片触发缓存清理

### 3.2 下载重试机制

- **D-04:** 重试策略 = 指数退避
  - 最多 3 次重试
  - 间隔：1s → 2s → 4s

- **D-05:** 重试触发条件 = 网络错误
  - 超时、连接失败：重试
  - 4xx 客户端错误：不重试
  - 5xx 服务器错误：重试
  - 取消/主动失败：不重试

- **D-06:** 具体改动位置 = `src-tauri/src/commands/download.rs`
  - 在 `start_download` 的 HTTP 请求外层包装重试循环
  - 用 `tokio::time::sleep` 实现退避

### 3.3 大目录扫描优化

- **D-07:** 优化方案 = 异步并行扫描
  - 改 `fs::read_dir` → `tokio::fs::read_dir`
  - 子目录并行扫描，汇总结果

- **D-08:** 并发控制 = Semaphore 限制
  - 并发上限 10 个目录句柄
  - 避免耗尽 OS 文件描述符

- **D-09:** 具体改动位置 = `src-tauri/src/commands/gallery.rs`
  - `get_directory_tree`、`get_local_images` 改异步

### 3.4 请求限流

- **D-10:** 限流方案 = 全局固定延迟
  - 每请求间隔 500ms
  - 硬编码，不暴露给用户设置

- **D-11:** 限流范围 = 全局
  - 覆盖 Gelbooru 搜索和图片下载所有 HTTP 请求
  - 在 `http.rs` 的 `HttpClient` 层统一注入

- **D-12:** 具体改动位置 = `src-tauri/src/services/http.rs`
  - 在 `HttpClient` 添加 `last_request_time: RwLock<Instant>`
  - 每次请求前 sleep 补足 500ms 间隔

## Prior Decisions (from Phase 2)

- 70% 覆盖率目标
- 使用 `#[cfg(test)]` 内联测试模块
- ESLint/Prettier + Husky pre-commit hook 已配置
- Rust 使用 `anyhow` 处理应用错误

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Context
- `.planning/phases/02-quality-testing/02-CONTEXT.md` — Phase 2 decisions
- `.planning/phases/02-quality-testing/02-SUMMARY.md` — Phase 2 completion report

### Project Docs
- `.planning/ROADMAP.md` § Phase 3 — Phase goal and task list
- `.planning/REQUIREMENTS.md` — Identified issues list (内存泄漏、重试、扫描、限流)
- `.planning/PROJECT.md` — Tech stack: Tauri 2.x, Vue 3, Rust

### Codebase Patterns
- `.planning/codebase/ARCHITECTURE.md` — Tauri IPC 通信模式、DownloadManager 架构
- `.planning/codebase/CONVENTIONS.md` — Code conventions
- `.planning/codebase/STRUCTURE.md` — File structure

### Rust Patterns
- `src-tauri/src/commands/download.rs` — DownloadManager 当前实现（无重试）
- `src-tauri/src/commands/gallery.rs` — `fs::read_dir` 同步扫描
- `src-tauri/src/services/http.rs` — `HttpClient` 当前实现（无限流）

### Frontend Patterns
- `src/views/Gallery.vue` — `preloadImages` 和 `imageBase64Cache` 使用位置
- `src/utils/lruCache.ts` — LRU 缓存实现

## Existing Code Insights

### Reusable Assets
- `LruCache<T>` (`src/utils/lruCache.ts`): 已实现，可复用
- `DownloadManager` (`download.rs`): 已有的任务管理结构，重试逻辑在其上封装
- `HttpClient` (`http.rs`): HTTP 客户端基础，中间件模式适合注入限流

### Established Patterns
- Tauri 命令返回 `Result<T, String>` 风格
- 前端用 `invoke` 调用 Rust 命令
- `tokio::spawn` 处理异步下载任务
- `mpsc::channel` 用于取消令牌

### Integration Points
- 重试逻辑：包裹 `start_download` 中 `http_client.download_image` 调用
- 限流逻辑：`HttpClient::get`/`download_image` 方法入口处注入延迟
- 懒加载：`Gallery.vue` 的 `preloadImages` 和 `handleImageError` 函数

## Specific Ideas

- IntersectionObserver API 用于前端懒加载（原生浏览器 API，无依赖）
- Semaphore 在 `tokio::sync` 中已有，可直接使用
- 限流 `sleep` 用 `tokio::time::sleep` 而非 `std::thread::sleep`（异步上下文兼容）

## Deferred Ideas

- 自适应限流（检测 429 自动降速）— Phase 3 固定 500ms，未来可探索
- 设置界面暴露限流参数 — Phase 3 固定，未来有需求再提
- 数据库缓存目录树结构 — 增量扫描优化，暂不进入 Phase 3 范围

---

*Phase: 03-performance-reliability*
*Context gathered: 2026-04-16*
