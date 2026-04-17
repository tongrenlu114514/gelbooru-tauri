# Gelbooru Downloader - Project Context

**Created:** 2026-04-14
**Status:** Brownfield - Existing Tauri application

## Overview

Gelbooru Downloader 是一个基于 Tauri 2.x 的桌面应用程序，用于从 Gelbooru 图片站搜索和下载图片。

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
- SQLite 本地持久化

## External Dependencies

- Gelbooru API (非官方，通过 HTML 抓取)
- 浏览器 Cookie 认证
- Tauri FS/Shell 插件

## Current State

Phase 03 (performance-reliability) complete — 220 tests passing (118 frontend Vitest + 102 Rust cargo), IntersectionObserver lazy loading, exponential backoff retry, async parallel directory scan, global HTTP rate limiting.

Last updated: 2026-04-17
