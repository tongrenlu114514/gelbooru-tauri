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

### Current Milestone: v1.2 Viewer & Indexing

**Goal:** 改进图片预览体验和图库索引性能

**Target features:**
- 图片查看器：全屏查看、缩放、拖拽、Lightroom 风格筛选
- 标签自动补全/推荐
- 下载暂停/恢复/重试体验
- 图库索引优化：搜索缓存、缩略图预生成

## Active Requirements (v1.2)

- UI-01: 图片查看器 UI
- UI-02: 缩放和拖拽交互
- UI-03: 键盘导航支持
- TAG-01: 标签自动补全
- TAG-02: 标签推荐
- DL-01: 暂停/恢复下载
- DL-02: 下载重试机制
- IDX-01: 图库索引缓存 ✅ Phase 10
- IDX-02: 缩略图预生成 ✅ Phase 10
- IDX-03: 按需缩略图生成 ✅ Phase 10
- IDX-04: 后台缩略图队列 ✅ Phase 10

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

### Out of Scope

- Mobile app — web-first approach
- Video chat — use external tools
- Offline mode — real-time is core value

## Context

- Phase 06 (waterfall-breadcrumb) complete — MasonryWall waterfall layout, NBreadcrumb navigation
- v1.1 UI milestone complete — 118 frontend tests, 102 Rust tests
- Schema version table for migration tracking
- v1.2 Started: Viewer & Indexing (图片查看器、标签增强、下载管理、索引优化)

Last updated: 2026-05-15 — Phase 10 (gallery-indexing) complete — IDX-01 through IDX-04 validated