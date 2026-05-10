# Phase 6: 瀑布流布局 + 面包屑导航 - Context

**Gathered:** 2026-05-10
**Status:** Ready for planning

<domain>
## Phase Boundary

Switch image grid from fixed-column CSS Grid to true masonry waterfall layout, replace flat path display with hierarchical breadcrumb navigation showing the folder hierarchy, enable click on any breadcrumb segment to jump to that ancestor folder and scroll to the first image of that folder visible in the viewport. Keep UI clean, responsive, and smooth.

</domain>

<decisions>
## Implementation Decisions

### 6.1 瀑布流布局

- **D-01:** 实现方案 = JS masonry 库（精确定位）
  - 使用 Vue 3 兼容的 masonry 库，如 `@yeger/vue-masonry-wall`
  - 不使用 CSS `column-count`（列优先顺序、滚动条抖动问题）
  - Masonry 行为：等宽列、高度不等自动堆叠，顺序按行优先

### 6.2 面包屑导航

- **D-02:** 数据来源 = 从图片文件路径解析面包屑段
  - 图片 `path` 字段已知（如 `C:/Users/Downloads/Gelbooru/ArtistA/image1.jpg`）
  - 解析相对路径：`ArtistA` → 上级 `Gelbooru` → 上级 `Downloads`
  - 不依赖"当前选中目录"的 parentPath，而是从图片自身路径反推层级
  - 面包屑段数量 = 截取到 downloadPath 后的路径段数

- **D-03:** 点击行为
  - 点击面包屑任一段 → 导航到该文件夹 + 滚动到该文件夹内**第一张图片在视口的坐标位置**
  - 定位逻辑：找到该文件夹的子目录/图片卡片中第一张图片的 DOM 位置，用 `scrollIntoView({ behavior: 'smooth', block: 'center' })` 滚动到视口中央
  - 如果文件夹已在当前视图内（可见），不触发滚动

- **D-04:** 文件夹切换滚动行为
  - 切换文件夹后，平滑滚动到该文件夹内第一张图片卡片位置
  - `scrollIntoView({ behavior: 'smooth', block: 'start' })`

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Context
- `.planning/phases/05-redesign-local-gallery/05-CONTEXT.md` — Phase 5 decisions (convertFileSrc, hover gradient, NSkeleton, card style)
- `.planning/phases/05-redesign-local-gallery/05-01-PLAN.md` — Phase 5 implementation plan
- `.planning/ROADMAP.md` § Phase 6 — Phase goal

### Project Docs
- `.planning/PROJECT.md` — Tech stack: Tauri 2.x, Vue 3, naive-ui, Pinia
- `.planning/PROJECT.md` § Current State — 220 tests passing, Phase 03 IntersectionObserver lazy loading preserved

### Codebase
- `src/views/Gallery.vue` — Current gallery implementation (flat folder list, path bar, grid)
- `src/views/GalleryCards.vue` — Current grid implementation (CSS Grid `auto-fill, minmax(160px, 1fr)`, NSkeleton, hover)
- `src/stores/settings.ts` — `downloadPath` for breadcrumb root resolution

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `GalleryCards.vue` — Current grid component, needs masonry refactor (keep props interface)
- `IntersectionObserver` — Phase 3 lazy loading, preserve during masonry transition
- `convertFileSrc` — Primary image URL, unchanged
- `settingsStore.downloadPath` — Used as breadcrumb root anchor

### Established Patterns
- Vue 3 Composition API + `<script setup lang="ts">`
- Pinia for state (`useSettingsStore`)
- naive-ui components (`NLayoutContent`, `NIcon`)
- Tauri IPC：`invoke` → `get_directory_images`, `get_directory_tree`

### Integration Points
- `get_directory_images` → returns `subdirs[]` + `images[]` with `path` fields (path data available)
- GalleryCards `class="content-grid"` → refactor to masonry container
- Path bar (`.path-bar`) → replace with breadcrumb component
- Folder list (`.folder-list`) → keep above masonry grid

</code_context>

<specifics>
## Specific Ideas

### Masonry Library Selection
- **@yeger/vue-masonry-wall** — Pure Vue 3, no jQuery, `width` prop, responsive, gap support
  - Install: `pnpm add @yeger/vue-masonry-wall`
  - Usage: `<MasonryWall :items="images" :column-width="160" :gap="4">` with scoped slot for rendering
  - SSR safe, tested with Vue 3.5+
  - Alternative: `vue-masonry` (older, less maintained) or native CSS column-count (not chosen)

### Breadcrumb Path Resolution
- Image path: `C:/Users/Downloads/Gelbooru/ArtistA/image1.jpg`
- `downloadPath`: `C:/Users/Downloads/Gelbooru/`
- Relative path: `ArtistA/image1.jpg`
- Breadcrumb segments: `Gelbooru > ArtistA` (from root of downloadPath to image's parent folder)
- Click `ArtistA` → `enterSubdir({ path: 'C:/Users/Downloads/Gelbooru/ArtistA', ... })` → scroll to first card in that folder

### Scroll-to-First-Image Logic
- After `enterSubdir(subdir)`, images in that subdir become visible in the masonry grid
- Use `nextTick` + `document.querySelector('[data-subdir-path="..."]')` to find the folder's first card
- `card.scrollIntoView({ behavior: 'smooth', block: 'start' })`

### Breadcrumb Display
- Show path segments from downloadPath root to current folder: `下载目录 > FolderA > FolderB`
- Each segment is clickable (navigate to that folder + scroll to first image of that folder)
- Current folder segment is non-clickable (already there)
- Use naive-ui `NBreadcrumb` + `NBreadcrumbItem` components

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 06-waterfall-breadcrumb*
*Context gathered: 2026-05-10*
*Masonry: @yeger/vue-masonry-wall | Breadcrumb: path-resolve | Scroll: smooth to first image*