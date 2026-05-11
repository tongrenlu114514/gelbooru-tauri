# Gelbooru Downloader - Project Context

**Created:** 2026-04-14
**Status:** Brownfield - Existing Tauri application

## Overview

Gelbooru Downloader 是一个基于 Tauri 2.x 的桌面应用程序，用于从 Gelbooru 图片站搜索和下载图片。

## Current State (v1.0 MVP Shipped)

**Milestone v1.0:** Complete (2026-05-10)
- Phase 1-4 completed with 12 plans
- Settings persistence to SQLite
- 80+ unit tests (frontend + backend)
- Performance optimizations (lazy loading, retry, rate limiting)

**Next Milestone:** v1.1 UI (Phase 5-6)
- Apple Photos gallery redesign
- Masonry waterfall layout
- Breadcrumb navigation

## Technology Stack

### Frontend
- Vue 3.5.x (Composition API)
- TypeScript 5.7.x
- Pinia (状态管理)
- Vue Router 4.x
- naive-ui 2.41.x
- Vite 6.x

### Backend (Rust)
- Tauri 2.x
- rusqlite (SQLite 数据库)
- reqwest (HTTP 客户端)
- scraper (HTML 解析)
- tokio (异步运行时)

## Key Features

1. **图片搜索** - 通过标签搜索 Gelbooru 图片
2. **图片下载** - 批量下载图片，支持并发控制
3. **本地图库** - 浏览本地已下载的图片
4. **收藏标签** - 收藏常用搜索标签
5. **设置管理** - 配置下载路径、代理等

## Architecture

- Tauri 多进程架构 (WebView + Rust)
- IPC 通信 (invoke/emit)
- Pinia 状态管理
- SQLite 本地持久化 (schema versioning enabled)

## External Dependencies

- Gelbooru API (非官方，通过 HTML 抓取)
- 浏览器 Cookie 认证
- Tauri FS/Shell 插件

## Requirements

### Validated (v1.0)

- Settings persistence — v1.0
- Download task restoration — v1.0
- Path traversal protection — v1.0
- Testing infrastructure (Vitest + Rust) — v1.0
- 80+ unit tests — v1.0
- Pre-commit hooks (Husky) — v1.0
- Image lazy loading — v1.0
- Download retry with backoff — v1.0
- Schema versioning — v1.0

### Active (v1.1)

- Apple Photos gallery redesign
- Masonry waterfall layout
- Breadcrumb navigation
- Keyboard shortcut support

### Out of Scope

- Mobile app — web-first approach
- Video chat — use external tools
- Offline mode — real-time is core value

## Context

- Phase 06 (waterfall-breadcrumb) complete — MasonryWall waterfall layout, NBreadcrumb navigation
- 118 frontend tests passing, 102 Rust tests passing
- Schema version table for migration tracking

Last updated: 2026-05-10 after v1.0 milestone