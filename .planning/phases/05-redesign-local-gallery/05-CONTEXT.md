---
phase: 5
phase_name: 重新设计本地图库显示界面
status: discussed
last_updated: "2026-05-08"
---

# Phase 5: 重新设计本地图库显示界面 - Context

**Gathered:** 2026-05-08
**Status:** Ready for planning

<domain>
## Phase Boundary

Redesign the local gallery display UI with an Apple Photos-inspired aesthetic. Scope covers layout structure, visual styling, information density, and loading states. No new capabilities — only UI/UX refinement of the existing gallery browsing feature.
</domain>

<decisions>
## Implementation Decisions

### 5.1 布局结构 — 双面板 + 可折叠侧边栏

- **D-01:** 布局模型 = 双面板（Side-by-side）
  - 左侧：文件夹树，侧边栏宽度固定 240px，可折叠
  - 右侧：图片网格区域，撑满剩余宽度
  - 侧边栏折叠后图片区无遮挡，类似 Apple Photos Cmd+\ 行为

- **D-02:** 侧边栏行为
  - 默认展开（240px 宽）
  - 支持手动折叠/展开（点击切换按钮）
  - 文件夹树结构不变（`get_directory_tree` 命令继续使用）

### 5.2 视觉风格 — Apple Photos 风格瀑布流

- **D-03:** 视觉语言 = Apple Photos 风格（统一卡片 + 固定列瀑布流）
  - 参考：Apple Photos macOS/iOS 相册网格
  - 特点：白底、无边框卡片、subtle 阴影、hover 遮罩、圆角统一

- **D-04:** 卡片样式
  - 背景：纯白或浅灰（`#fafafa`），无边框
  - 圆角：4px（与 Apple Photos 一致）
  - 间距：2-4px（紧凑，减少留白）
  - 阴影：仅 hover 时 `box-shadow: 0 2px 8px rgba(0,0,0,0.15)`
  - 瀑布流列数：桌面端 4-6 列，平板 3-4 列，移动端 2-3 列

- **D-05:** 图片来源显示方式
  - 使用 `convertFileSrc` 直连 Tauri asset URL（不依赖 base64 缓存）
  - `imageBase64Cache` 作为 fallback 仅在 `convertFileSrc` 失败时使用

### 5.3 信息密度 — 隐藏文字，hover 显示

- **D-06:** 默认状态 = 纯图片，无文字标签
  - 文件名、尺寸、日期均不在卡片上显示
  - 保持最大图片显示面积，减少视觉噪音

- **D-07:** Hover 状态 = 底部渐变遮罩 + 文件名
  - 底部渐变遮罩（`linear-gradient(transparent, rgba(0,0,0,0.6))`）
  - 底部显示：文件名（单行截断）
  - 无操作按钮（与 Apple Photos 一致，点击打开预览）
  - 选中状态：左上角蓝色勾选框

- **D-08:** 预览行为
  - 点击卡片 → 打开 Modal 全屏预览（保留 ArrowLeft/Right 导航）
  - 预览中显示文件名 + 图片数量（1/42 格式）

### 5.4 空状态 & 加载状态 — 骨架屏

- **D-09:** 加载状态 = 骨架屏（Skeleton cards）代替 NSpin
  - 灰色脉冲占位符卡片，3×3 或 4×4 网格
  - 骨架数量与预期图片数量匹配（加载前估算）

- **D-10:** 空目录状态 = 居中图标 + 引导文案
  - 居中显示文件夹图标（NIcon: FolderOutline，size 64）
  - 文案："该目录下暂无图片"（NEmpty）
  - 不使用插画，保持简洁

### 5.5 文件夹卡片

- **D-11:** 文件夹卡片风格与图片卡片统一
  - 保持文件夹图标 + 名称 + 计数（文件夹卡片需显示信息以区分）
  - 缩略图优先（子目录第一张图片），无缩略图时显示文件夹图标

### Claude's Discretion

- 瀑布流列数具体数值由 planner 根据视口宽度计算
- 骨架屏动画（pulse/translucent）由 planner 选择 naive-ui 方案
- 侧边栏折叠动画（transition width）

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Context
- `.planning/phases/02-quality-testing/02-CONTEXT.md` — Project test coverage requirements
- `.planning/phases/03-performance-reliability/03-CONTEXT.md` — Gallery.vue current implementation (IntersectionObserver, imageCache)
- `.planning/phases/04-polish-release/04-CONTEXT.md` — Recent changes
- `.planning/ROADMAP.md` § Phase 5 — Phase goal

### Project Docs
- `.planning/PROJECT.md` — Tech stack: Tauri 2.x, Vue 3, naive-ui, Pinia
- `.planning/REQUIREMENTS.md` — Identified issues

### Codebase Patterns
- `src/views/Gallery.vue` — Current gallery implementation (tree sidebar + grid) — **READ FIRST**
- `src/stores/settings.ts` — Settings store (download path, watcher)
- `src/utils/lruCache.ts` — LruCache for base64 fallback
- `src-tauri/src/commands/gallery.rs` — `get_directory_tree`, `get_directory_images` commands

### Design Reference
- Apple Photos (macOS/iOS) — Layout reference: two-panel sidebar + grid, collapsible sidebar
- naive-ui component library — Use `NSkeleton`, `NEmpty`, `NLayout`, `NLayoutSider` for implementation

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `NLayout` + `NLayoutSider` (`naive-ui`): Already in use, extend for collapsible behavior
- `IntersectionObserver` (`Gallery.vue`): Already set up for lazy loading — reuse for hover overlay
- `LruCache<string, string>` (`lruCache.ts`): Reuse as base64 fallback only
- `convertFileSrc` (`@tauri-apps/api/core`): Primary image URL method

### Established Patterns
- Vue 3 Composition API + `<script setup lang="ts">`
- Pinia stores for state (`useSettingsStore`)
- Tauri IPC：`invoke` → Rust commands
- naive-ui components for UI shell

### Integration Points
- `get_directory_tree` → TreeNode sidebar (unchanged command, changed rendering)
- `get_directory_images` → Grid data (unchanged command, changed rendering)
- Sidebar collapse state → local component state (no store needed)
- Hover overlay → CSS `:hover` + IntersectionObserver for dynamic

</code_context>

<specifics>
## Specific Ideas

### Apple Photos Design Reference
- **Layout**: `NLayout` with `NLayoutSider` (collapsible, 240px default)
- **Grid**: CSS Grid `repeat(auto-fill, minmax(160px, 1fr))` with `gap: 4px`
- **Cards**: White background `#fff`, `border-radius: 4px`, no border, `overflow: hidden`
- **Hover**: `::after` overlay `background: linear-gradient(transparent 50%, rgba(0,0,0,0.6))` + filename text
- **Selection**: `::before` blue checkbox top-left on `.selected` class
- **Skeleton**: `n-skeleton` components in 4×4 grid, animated
- **Preview modal**: Current `NModal` + `ChevronLeft/Right` + `TrashOutline` keep as-is
- **Sidebar collapse**: Add toggle button, `collapsed` state, `collapsed-width: 0`

### Ref vs Masonry
用户选择了 Apple Photos 风格，该风格实际上是**固定列瀑布流**（uniform grid），不是真正的 masonry。
- 固定列数：`auto-fill, minmax(160px, 1fr)` → 列数随视口变化，列内图片等宽
- 高度不等时，行高由该列最高图片决定（标准 CSS Grid 行为）

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 05-redesign-local-gallery*
*Context gathered: 2026-05-08*
*Reference: Apple Photos aesthetic*