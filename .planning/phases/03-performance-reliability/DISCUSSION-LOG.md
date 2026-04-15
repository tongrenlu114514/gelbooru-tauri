# Phase 3: Performance & Reliability - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-16
**Phase:** 03-performance-reliability
**Areas discussed:** imageCache 内存泄漏, 下载重试机制, 大目录扫描优化, 请求限流

---

## imageCache 内存泄漏

| Option | Description | Selected |
|--------|-------------|----------|
| LRU 缓存大小不足 | 100 条不够，扩大缓存或改用 convertFileSrc | |
| convertFileSrc 路径不缓存 | 两种都缓存 | |
| 预加载失控 | preloadImages 遍历所有路径 → 懒加载 + 缓存 | ✓ |

**User's choice:** 预加载失控
**Notes:** `preloadImages` 无限制遍历所有图片路径导致 base64 内存暴涨

| Option | Description | Selected |
|--------|-------------|----------|
| 懒加载 + 缓存 | 只加载屏幕内可见，屏幕外清除 | ✓ |
| 分页预加载 | 预加载当前页，切换清空 | |
| 可见区域预加载 | 加载屏幕内 + 上下 N 张 | |

**User's choice:** 懒加载 + 缓存

---

## 下载重试机制

| Option | Description | Selected |
|--------|-------------|----------|
| 指数退避 | 最多 3 次，1s → 2s → 4s | ✓ |
| 固定间隔 | 最多 3 次，固定 2s | |
| 仅网络错误重试 | 只对超时不重试，最保守 | |

**User's choice:** 指数退避

| Option | Description | Selected |
|--------|-------------|----------|
| 网络错误 | 超时/连接失败重试；4xx 不重试；5xx 重试 | ✓ |
| 所有错误 | 所有非取消错误都重试 | |
| 超时+5xx | 超时和 5xx 重试 | |

**User's choice:** 网络错误

---

## 大目录扫描优化

| Option | Description | Selected |
|--------|-------------|----------|
| 异步并行 | 改 tokio::fs::read_dir 并行扫描子目录 | ✓ |
| 增量缓存 | 扫描结果缓存数据库，增量更新 | |
| 分页加载 | 只返回前 N 张，滚动时继续加载 | |

**User's choice:** 异步并行

| Option | Description | Selected |
|--------|-------------|----------|
| Semaphore 限制并发 | 并发上限 10 个目录句柄 | ✓ |
| 无限制并行 | 直接并行，可能耗尽资源 | |

**User's choice:** Semaphore 限制并发

---

## 请求限流

| Option | Description | Selected |
|--------|-------------|----------|
| 全局延迟 | 每请求间隔 500ms | ✓ |
| 自适应限流 | 检测 429 自动降速 | |
| 分场景限流 | 搜索 1s，下载不限 | |

**User's choice:** 全局延迟

| Option | Description | Selected |
|--------|-------------|----------|
| 固定值 | 硬编码 500ms，不暴露设置 | ✓ |
| 设置项暴露 | 在设置界面可调 100ms ~ 2s | |

**User's choice:** 固定值

---

## Claude's Discretion

- LRU 缓存大小维持 100 条不变（懒加载后不会持续增长）
- 重试逻辑在 `start_download` 命令层封装，不修改 `HttpClient` 核心
- 目录扫描并发上限 10 为经验值，可在执行时根据实际情况调整

## Deferred Ideas

- 自适应限流（检测 429 动态调整延迟）— 暂不需要，Phase 3 固定 500ms
- 设置界面暴露限流参数 — Phase 3 固定，用户无此需求
- 数据库缓存目录树结构 — 超前优化，暂不需要
