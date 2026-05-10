# Phase 6: 瀑布流布局 + 面包屑导航 - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-10
**Phase:** 06-waterfall-breadcrumb
**Areas discussed:** 瀑布流实现方案, 面包屑数据来源, 面包屑点击行为, 滚动位置保持

---

## 瀑布流实现方案

| Option | Description | Selected |
|--------|-------------|----------|
| CSS `column-count` | 简单，无依赖，天然瀑布流（列内顺序排列） | |
| JS masonry 库 | 更精确的定位，额外依赖 | ✓ |

**User's choice:** JS masonry 库 — 更精确的定位
**Notes:** 选中精确定位，不使用 CSS column-count（列优先顺序、滚动条抖动问题）

---

## 面包屑数据来源

| Option | Description | Selected |
|--------|-------------|----------|
| 从当前目录路径自动解析 | 路径已知，直接拆 segment 生成 | |
| 从图片文件名反推路径段 | 更灵活，需要额外数据 | ✓ |

**User's choice:** 从图片文件名反推路径段
**Notes:** 图片 path 字段已知，解析相对路径提取层级，不依赖"当前选中目录"

---

## 面包屑点击行为

| Option | Description | Selected |
|--------|-------------|----------|
| 跳转到该文件夹 + 滚动到该文件夹卡片位置 | 滚动到 DOM 位置 | |
| 跳转到该文件夹 + 滚动到该文件夹内第一张图片在视口的坐标位置 | 平滑滚动到视口中央 | ✓ |

**User's choice:** 跳转到该文件夹 + 滚动到该文件夹内第一张图片在视口的坐标位置
**Notes:** `scrollIntoView({ behavior: 'smooth', block: 'center' })` 滚动到视口中央

---

## 滚动位置保持

| Option | Description | Selected |
|--------|-------------|----------|
| 切换文件夹后，滚动到顶部 | 当前默认行为 | |
| 切换文件夹后，滚动到第一个图片卡片位置 | 平滑滚动 | ✓ |

**User's choice:** 切换文件夹后，滚动到第一个图片卡片位置（平滑滚动）
**Notes:** `scrollIntoView({ behavior: 'smooth', block: 'start' })`

---

## Claude's Discretion

- Masonry 库具体版本和 API 由 planner 选择（@yeger/vue-masonry-wall 备选）
- 面包屑组件实现（NBreadcrumb vs 自定义）由 planner 选择
- 面包屑根路径锚定基于 `settingsStore.downloadPath`

## Deferred Ideas

None — discussion stayed within phase scope.