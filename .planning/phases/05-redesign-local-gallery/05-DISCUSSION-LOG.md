# Phase 5: 重新设计本地图库显示界面 - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-08
**Phase:** 05-redesign-local-gallery
**Areas discussed:** 布局结构, 视觉风格, 信息密度, 空状态/加载态

---

## 布局结构

| Option | Description | Selected |
|--------|-------------|----------|
| 方案 A：保持双面板（树+网格），侧边栏可折叠 | 稳态，灵活性高 | ✓ |
| 方案 B：左侧改为图片缩略图条（filmstrip） | 图片量大时效率高 | |
| 方案 C：左侧折叠为图标导航（收藏夹快速跳转），网格撑满 | 最大化图片显示面积 | |

**User's choice:** 方案 A：保持双面板（树+网格），侧边栏可折叠，参考苹果相册风格

---

## 视觉风格

| Option | Description | Selected |
|--------|-------------|----------|
| 方案 A：统一卡片（瀑布流），Apple Photos 风格 | 图片尺寸多样时效果好 | ✓ |
| 方案 B：大图模式（无网格间距，点击切换全屏浏览） | 以单张浏览为主 | |
| 方案 C：密集网格（缩小卡片，增加每屏数量，间距 4px） | 图片量大时效率高 | |
| 方案 D：保持当前风格（微调细节，不做架构级改动） | 够用就行 | |

**User's choice:** 方案 A：统一卡片风格，瀑布流布局，参考 Apple Photos

---

## 信息密度

| Option | Description | Selected |
|--------|-------------|----------|
| 文件名（当前） | 显示文件名 | |
| 文件大小（KB/MB） | 显示文件大小 | |
| 图片尺寸（1920×1080） | 显示图片分辨率 | |
| 下载日期 | 显示下载日期 | |
| 图片评分/标签（如果有） | 显示评分标签 | |
| 完全隐藏文字，hover 才有 | 纯图片无文字，hover 显示 | ✓ |

**User's choice:** 完全隐藏所有文字，hover 时才显示文件名和操作按钮

---

## 空状态/加载态

| Option | Description | Selected |
|--------|-------------|----------|
| 方案 A：保持朴素（NEmpty + NSpin） | 够用 | |
| 方案 B：空目录显示插画 + 引导文案 | 有品味 | |
| 方案 C：加载骨架屏（Skeleton cards 代替 NSpin） | 现代感，用户体验好 | ✓ |
| 方案 D：空目录直接显示拖入区域（支持本地导入） | 新功能，范围外 | |

**User's choice:** 方案 C：加载骨架屏（Skeleton cards 代替 NSpin）

---

## Summary

All 4 gray areas resolved in a single pass. User chose:
1. **布局**: 双面板 + 可折叠侧边栏（Apple Photos 风格）
2. **视觉**: 统一卡片 + 固定列瀑布流（Apple Photos 风格）
3. **信息**: 纯图片无文字，hover 显示文件名
4. **加载**: Skeleton cards 代替 NSpin

Reference aesthetic: **Apple Photos (macOS/iOS)** — uniform grid, white background, no borders, hover overlay with gradient.