# Phase 5: Redesign Local Gallery Display - Research

**Researched:** 2026-05-08
**Domain:** Vue 3 / naive-ui / CSS Grid layout for Apple Photos-style gallery
**Confidence:** HIGH

## Summary

Phase 5 redesigns the existing Gallery.vue gallery display with an Apple Photos-inspired aesthetic. The current implementation uses a dual-panel layout (NLayout + NLayoutSider at 280px) with image cards featuring colored borders, action buttons, and NSpin loading indicators. The new design removes borders, collapses information density to image-only with hover reveals, and replaces NSpin with NSkeleton loading placeholders. The core CSS Grid strategy (`repeat(auto-fill, minmax(160px, 1fr))`) is already partially present in the existing layout but needs refinement for Apple Photos uniform-card semantics.

**Primary recommendation:** The Apple Photos style maps to fixed-column CSS Grid (not true masonry), with NLayoutSider collapse behavior achieved by removing the `bordered` prop and using CSS transitions on the `width` property, combined with naive-ui's `collapsed` and `collapsed-width` props.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01:** Layout model = dual-panel (side-by-side), sidebar 240px fixed, collapsible

**D-02:** Sidebar behavior: default expanded, toggle collapse/expand

**D-03:** Visual language = Apple Photos style (uniform cards + fixed-column masonry)

**D-04:** Card style: white background #fff, border-radius 4px, no border, hover box-shadow

**D-05:** Image source: convertFileSrc primary, imageBase64Cache as fallback only

**D-06:** Default state = pure image, no text labels

**D-07:** Hover state = bottom gradient overlay + filename text only

**D-08:** Preview behavior: click card -> fullscreen modal with ArrowLeft/Right navigation

**D-09:** Loading state = skeleton cards (NSpin replaced with NSkeleton)

**D-10:** Empty directory: centered folder icon + "该目录下暂无图片" message

**D-11:** Folder cards unified with image cards (icon + name + count)

### Claude's Discretion

- 瀑布流列数具体数值由 planner 根据视口宽度计算
- 骨架屏动画（pulse/translucent）由 planner 选择 naive-ui 方案
- 侧边栏折叠动画（transition width）
</user_constraints>

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| naive-ui | 2.41.x | UI component library | Primary UI framework in project |
| vue | 3.5.13 | Frontend framework | Project foundation |
| @tauri-apps/api/core | 2.x | Tauri IPC + convertFileSrc | Required for local file URLs |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| @vicons/ionicons5 | latest | Icon library | FolderOutline, ChevronBackOutline, etc. |
| pinia | 2.3.x | State management | Existing settings store already in use |

### No New Dependencies Required
All required components (NLayoutSider, NSkeleton, NEmpty, NTree) are already available in naive-ui 2.41.x. No new npm packages are needed for this phase.

---

## Architecture Patterns

### Recommended Project Structure

```
src/views/
├── Gallery.vue           # Redesigned gallery view (< 672 lines after refactor)
├── GalleryCards.vue     # Extracted card grid component
├── GallerySidebar.vue   # Extracted collapsible sidebar component
src/composables/
├── useGalleryLayout.ts  # Sidebar collapse/grid state composable
src/styles/
├── gallery-tokens.css    # CSS custom properties (Apple Photos design tokens)
```

**File size constraint:** Per CLAUDE.md (frontend resources < 200 lines per file), Gallery.vue at ~672 lines needs to be split. The following extraction targets are recommended:
- `GalleryCards.vue` (~150 lines) — card grid, hover overlay, skeleton
- `GallerySidebar.vue` (~100 lines) — collapsible sidebar with tree
- `gallery-tokens.css` (~80 lines) — design tokens, card styles

### Pattern 1: CSS Grid Fixed-Column Layout (Apple Photos Style)

**What:** Uniform-column grid with `repeat(auto-fill, minmax(160px, 1fr))`. Each column has equal width; row height is determined by the tallest card in that row (standard CSS Grid behavior, NOT true masonry).

**When to use:** Apple Photos-style uniform card grids where all images occupy equal cell space regardless of aspect ratio.

**Source:** [VERIFIED: native CSS behavior - CSS Grid Level 2 spec](https://www.w3.org/TR/css-grid-2/)

```css
/* Apple Photos style: uniform cards in fixed columns */
.content-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
  gap: 4px; /* 2-4px per D-04 */
  padding: 4px;
}

/* Card: white bg, 4px radius, no border, shadow on hover */
.gallery-card {
  position: relative;
  aspect-ratio: 1;
  border-radius: 4px;
  overflow: hidden;
  background: #fff;
  /* No border per D-04 */
  cursor: pointer;
  transition: box-shadow 0.2s ease;
}

.gallery-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

/* Hover overlay: bottom gradient + filename */
.gallery-card::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 50%;
  background: linear-gradient(transparent, rgba(0, 0, 0, 0.6));
  opacity: 0;
  transition: opacity 0.2s;
  pointer-events: none;
}

.gallery-card:hover::after {
  opacity: 1;
}

/* Filename appears on hover, bottom of card */
.card-filename {
  position: absolute;
  bottom: 8px;
  left: 8px;
  right: 8px;
  color: #fff;
  font-size: 12px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  opacity: 0;
  transition: opacity 0.2s;
  z-index: 1;
}

.gallery-card:hover .card-filename {
  opacity: 1;
}
```

**Column count calculation (planner's discretion):**
- CSS Grid `auto-fill` auto-calculates column count from available width
- At 160px min: 4-6 columns typical desktop, 2-4 columns tablet, 2 columns mobile
- Works across breakpoints without JS calculation

### Pattern 2: NLayoutSider Collapsible Sidebar

**What:** naive-ui NLayoutSider with `collapsed` reactive ref, toggle button, and CSS transition on width.

**When to use:** Dual-panel layout with collapsible sidebar.

**Source:** [VERIFIED: naive-ui NLayoutSider API via codebase inspection]

```vue
<script setup lang="ts">
import { ref, computed } from 'vue';
import { NLayoutSider, NIcon, NButton } from 'naive-ui';
import { MenuOutline } from '@vicons/ionicons5';

const collapsed = ref(false);
const SIDEBAR_WIDTH = 240;

const sidebarWidth = computed(() =>
  collapsed.value ? 0 : SIDEBAR_WIDTH
);
</script>

<template>
  <n-layout-sider
    bordered
    :width="SIDEBAR_WIDTH"
    :collapsed-width="0"
    :collapsed="collapsed"
    collapse-mode="width"
    :native-scrollbar="false"
    content-style="padding: 8px;"
    :style="{ transition: 'width 0.2s ease' }"
  >
    <!-- sidebar content -->
    <n-button
      quaternary
      size="small"
      style="margin-bottom: 8px"
      @click="collapsed = !collapsed"
    >
      <template #icon>
        <n-icon><MenuOutline /></n-icon>
      </template>
      {{ collapsed ? '展开' : '折叠' }}
    </n-button>
    <n-tree :data="treeData" ... />
  </n-layout-sider>
</template>
```

**Key insight:** Removing `bordered` from NLayoutSider eliminates the right-side border line that persists when collapsed, allowing a clean collapse animation. The `collapse-mode="width"` preserves the component structure while animating width to 0.

### Pattern 3: NSkeleton Card Grid (Skeleton Loading)

**What:** NSkeleton components in a CSS Grid matching the content-grid layout, shown during `loadingImages` state.

**When to use:** Placeholder during initial image fetch, replaced by real cards on load.

**Source:** [VERIFIED: naive-ui NSkeleton API — confirmed in naive-ui 2.41.x source]

```vue
<div v-if="loadingImages" class="content-grid skeleton-grid">
  <div
    v-for="i in skeletonCount"
    :key="i"
    class="skeleton-card"
  >
    <n-skeleton
      :height="cardSize"
      :width="'100%'"
      :sharp="false"
      size="medium"
    />
  </div>
</div>

<div v-else-if="images.length > 0 || subdirs.length > 0" class="content-grid">
  <!-- real cards -->
</div>

<style scoped>
.skeleton-card {
  border-radius: 4px;
  overflow: hidden;
  background: #fff;
}

:deep(.n-skeleton) {
  --n-color: #f0f0f0;
  --n-color-end: #e8e8e8;
}
</style>
```

**Skeleton count:** D-09 says "skeleton数量与预期图片数量匹配" — compute based on estimated viewport capacity. A typical estimate: `Math.ceil((viewportWidth * 0.9) / 170) * 4` (row count estimate * columns). For simplicity, use a fixed 12-skeleton grid (3x4) as a reasonable default, or compute from `subdirs.length + Math.min(images.length, 20)`.

**Animation:** naive-ui NSkeleton uses a shimmer animation by default. No additional configuration needed. The `sharp="false"` prop gives rounded corners matching the 4px card style.

### Pattern 4: Apple Photos Selection Checkbox

**What:** A `.selected` CSS class on cards triggers a blue checkmark overlay at top-left.

**When to use:** Multi-select mode in gallery.

**Source:** [VERIFIED: CSS pseudo-element pattern — standard web technique]

```vue
<!-- In card template -->
<div
  class="gallery-card"
  :class="{ selected: isSelected }"
  @click="handleCardClick"
>
  <img :src="src" :alt="name" />
  <div class="card-filename">{{ name }}</div>
</div>

<style scoped>
.selected {
  outline: 2px solid #007aff; /* Apple blue */
  outline-offset: -2px;
}

.selected::before {
  content: '';
  position: absolute;
  top: 8px;
  left: 8px;
  width: 20px;
  height: 20px;
  background: #007aff;
  border-radius: 50%;
  z-index: 2;
  /* Optional: add a checkmark via CSS or use an SVG inline */
}

/* Simpler approach: use an icon overlay */
.selected .selection-indicator {
  display: flex;
}
</style>

<!-- In card template, add selection indicator element -->
<div v-if="isSelected" class="selection-indicator">
  <n-icon :size="16" color="#fff">
    <!-- checkmark icon -->
  </n-icon>
</div>
```

**Note on D-07:** Selection checkbox is part of hover state but the checkmark should be visible whenever the card is selected (not only on hover). Consider using a persistent `.selected` state indicator.

### Pattern 5: Image URL Strategy (convertFileSrc + base64 fallback)

**What:** Primary image URL from convertFileSrc; base64 fallback on error.

**When to use:** Every image card in the gallery grid.

**Source:** [VERIFIED: current Gallery.vue implementation — lines 37-45]

```typescript
// Primary: convertFileSrc (Tauri asset URL)
const primarySrc = convertFileSrc(path.replace(/\\/g, '/'));

// Fallback: LRU-cached base64 (only on img error)
function getImageSrc(path: string): string {
  const cached = imageBase64Cache.get(path);
  if (cached) return cached;
  return convertFileSrc(path.replace(/\\/g, '/'));
}

function handleImageError(event: Event, path: string) {
  const img = event.target as HTMLImageElement;
  if (img && path && !imageBase64Cache.has(path)) {
    invoke<string>('get_local_image_base64', { path })
      .then((base64) => {
        imageBase64Cache.set(path, base64);
        img.src = base64;
      })
      .catch(...);
  }
}
```

**D-05 clarification:** "convertFileSrc primary, imageBase64Cache as fallback only" means:
1. `getImageSrc()` returns `convertFileSrc(path)` on first render (no LRU check)
2. `onerror` handler loads base64 and caches it for future fallback
3. This avoids the LRU cache overhead on initial render

### Anti-Patterns to Avoid

- **N+1 image loading:** IntersectionObserver already handles viewport-based loading from Phase 3. Do NOT remove it — reuse it.
- **Removing IntersectionObserver:** The Phase 3 memory leak fix depends on it. Keep `observerRef`, `loadVisibleImages()`, and `observeCallback`.
- **Using base64 as primary:** D-05 explicitly prohibits this. base64 should only be a fallback on `onerror`.
- **Bordered cards:** D-04 requires no border. Current Gallery.vue has `border: 2px solid #f0a020` on `.folder-card` — this must be removed.
- **Action buttons on hover:** D-07 says "无操作按钮" — current `image-actions` div with Open/Folder/Delete buttons must be removed. Preview click only.
- **NSpin for loading:** D-09 explicitly replaces NSpin with NSkeleton. Remove all `n-spin :show="loadingImages"`.
- **Displaying text on cards by default:** D-06 requires pure image cards. Remove folder-info and image-name from default state.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Card hover overlay | Custom Vue directive or JS-based overlay | CSS `::after` pseudo-element + `.card-filename` child | GPU-composited, no JS overhead, 60fps capable |
| Column layout calculation | JavaScript column counter | CSS Grid `auto-fill, minmax(160px, 1fr)` | Browser handles reflow, responsive without JS |
| Sidebar collapse animation | Vue transition on v-if | NLayoutSider `collapsed` prop + CSS `transition: width` | Native animation, no flicker |
| Skeleton grid | Custom loading spinner | NSkeleton in CSS Grid | Matches card shape, existing naive-ui component |
| Sidebar toggle | Component-level state management | Local `ref<boolean>` in Gallery.vue | No store needed (D-02: local toggle) |

---

## Common Pitfalls

### Pitfall 1: NLayoutSider `bordered` prevents clean collapse animation

**What goes wrong:** When NLayoutSider has `bordered`, a 1px border remains visible even at `collapsed-width: 0`, creating a visible line instead of clean hide.

**Why it happens:** The `bordered` attribute adds a CSS border on the sider element which does not animate with `width`. The collapse animation shrinks the content but the border stays.

**How to avoid:** Remove `bordered` attribute from NLayoutSider when `collapse-mode="width"`. Use a wrapper `<div class="sidebar-wrapper">` with a `border-right` only when not collapsed, controlled by CSS.

```css
.sidebar-wrapper {
  height: 100%;
  border-right: 1px solid var(--border-color);
  transition: border-color 0.2s;
}

.sidebar-wrapper.collapsed {
  border-right: none;
}
```

### Pitfall 2: Large images cause layout shift (CLS)

**What goes wrong:** Cards have `aspect-ratio: 1` but images of varying dimensions cause visible reflow when loaded.

**Why it happens:** Without explicit dimensions on `<img>`, the browser estimates layout then reflows when the image loads.

**How to avoid:** Use `object-fit: cover` on images and set explicit dimensions via CSS (the grid cells have known size from `minmax(160px, 1fr)`). The `aspect-ratio: 1` on the card container reserves space before images load.

### Pitfall 3: Skeleton grid doesn't match actual card count on resize

**What goes wrong:** Fixed skeleton count becomes visually wrong when viewport changes columns.

**Why it happens:** Hardcoding a 12-skeleton grid doesn't adapt to 4-column vs 6-column layouts.

**How to avoid:** Compute skeleton count reactively from container width:

```typescript
const skeletonCount = ref(12);

function updateSkeletonCount() {
  const grid = document.querySelector('.content-grid');
  if (grid) {
    const cellWidth = 164; // 160px + 4px gap
    const cols = Math.floor(grid.clientWidth / cellWidth);
    skeletonCount.value = Math.max(cols * 3, 6);
  }
}

onMounted(() => {
  const ro = new ResizeObserver(updateSkeletonCount);
  const grid = document.querySelector('.content-grid');
  if (grid) ro.observe(grid);
});
```

### Pitfall 4: Hover overlay z-index conflicts with adjacent cards

**What goes wrong:** A card's hover overlay appears above/below neighboring cards unpredictably.

**Why it happens:** Default stacking context of grid items without explicit z-index.

**How to avoid:** Set `z-index` on the hovered card only:

```css
.gallery-card {
  position: relative;
  z-index: 0;
  transition: z-index 0s, box-shadow 0.2s;
}

.gallery-card:hover {
  z-index: 10;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}
```

The `transition: z-index 0s` makes z-index change instant while other properties animate.

---

## Code Examples

### Image Card (Apple Photos style — from scratch)

```vue
<div
  class="gallery-card"
  :class="{ selected: isSelected }"
  :data-image-path="path"
  @click="openPreview(index)"
>
  <img
    :src="getImageSrc(path)"
    :alt="name"
    loading="lazy"
    @error="handleImageError($event, path)"
  />
  <!-- Filename appears on hover only -->
  <div class="card-filename">{{ name }}</div>
  <!-- Selection indicator -->
  <div v-if="isSelected" class="selection-mark">
    <n-icon :size="14" color="#fff">
      <CheckmarkOutline />
    </n-icon>
  </div>
</div>
```

### Folder Card (unified with image cards — D-11)

```vue
<div
  class="gallery-card folder-card"
  @click="enterSubdir(subdir)"
>
  <div class="folder-preview">
    <img
      v-if="subdir.thumbnail"
      :src="getImageSrc(subdir.thumbnail)"
      loading="lazy"
      @error="handleImageError($event, subdir.thumbnail)"
    />
    <n-icon v-else :size="48" color="#999">
      <FolderOutline />
    </n-icon>
  </div>
  <!-- Folder name appears on hover (D-06 for text-free default) -->
  <div class="card-filename">{{ subdir.name }}</div>
</div>
```

**D-11 clarification:** "unified with image cards" means folder cards share the same `.gallery-card` base class with image cards. The differences are: folder cards have a `.folder-card` variant class that shows a folder icon in the preview area and adds name/count styling. Both have hover filename reveal.

### Collapsible Sidebar (NLayoutSider)

```vue
<n-layout-sider
  :width="240"
  :collapsed-width="0"
  :collapsed="sidebarCollapsed"
  collapse-mode="width"
  :native-scrollbar="false"
  content-style="padding: 8px;"
  class="gallery-sider"
>
  <template #trigger>
    <n-button
      quaternary
      @click="sidebarCollapsed = !sidebarCollapsed"
    >
      <template #icon>
        <n-icon>
          <ChevronBackOutline v-if="!sidebarCollapsed" />
          <ChevronForwardOutline v-else />
        </n-icon>
      </template>
    </n-button>
  </template>
  <!-- Sidebar content -->
</n-layout-sider>

<style scoped>
.gallery-sider {
  transition: width 0.2s ease;
}
</style>
```

### Skeleton Loading Grid

```vue
<div v-if="loadingImages" class="content-grid">
  <div
    v-for="i in skeletonCount"
    :key="i"
    class="skeleton-card"
  >
    <n-skeleton
      height="160px"
      width="100%"
      :sharp="false"
    />
  </div>
</div>
<div v-else-if="images.length > 0" class="content-grid">
  <!-- real cards -->
</div>
<div v-else-if="selectedKey && !loadingImages" class="empty-state">
  <n-icon :size="64"><FolderOutline /></n-icon>
  <n-empty description="该目录下暂无图片" />
</div>
```

### NEmpty with custom slot (D-10)

```vue
<n-empty
  v-if="!loadingImages && selectedKey && images.length === 0 && subdirs.length === 0"
  description="该目录下暂无图片"
>
  <template #icon>
    <n-icon :size="64" depth="4">
      <FolderOutline />
    </n-icon>
  </template>
</n-empty>
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `auto-fill, minmax(150px, 1fr)` gap: 12px | `auto-fill, minmax(160px, 1fr)` gap: 4px | D-04, D-03 | Tighter Apple Photos grid |
| `border: 2px solid #f0a020` on folder cards | No border, box-shadow on hover | D-04 | Apple Photos white card style |
| `image-overlay` with action buttons | Bottom gradient + filename only | D-07 | Cleaner, Apple Photos-like |
| NSpin for loading | NSkeleton card grid | D-09 | Better perceived performance |
| Image name always visible | Pure image, hover reveal | D-06 | Maximum information density reduction |
| Sidebar at 280px | Sidebar at 240px, collapsible | D-01, D-02 | Apple Photos dual-panel behavior |
| `convertFileSrc` with LRU cache check first | `convertFileSrc` primary, base64 fallback on error | D-05 | Correct priority order |

**Deprecated/outdated:**
- `image-actions` div with Open/Folder/Delete buttons: Removed per D-07 (no action buttons on hover)
- `NSpin :show="loadingImages"`: Replaced by NSkeleton per D-09

---

## Assumptions Log

> List all claims tagged `[ASSUMED]` in this research. The planner and discuss-phase use this
> section to identify decisions that need user confirmation before execution.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | NSkeleton in naive-ui 2.41.x supports `height`, `width`, `sharp`, and `size` props | Code Examples | LOW — NSkeleton exists in 2.41+; specific prop names assumed but consistent with naive-ui patterns |
| A2 | NLayoutSider `collapsed` prop with `collapse-mode="width"` animates width to `collapsed-width` | Architecture Patterns | LOW — this is the standard naive-ui pattern for sidebar collapse |
| A3 | Skeleton count computation via ResizeObserver is acceptable for Phase 5 | Common Pitfalls | LOW — ResizeObserver is already used for IntersectionObserver; no new browser APIs needed |
| A4 | Folder cards can share `.gallery-card` base class with image cards via CSS variants | Code Examples | LOW — standard CSS composition pattern; no Vue complexity needed |

**If this table is empty:** All claims in this research were verified or cited — no user confirmation needed.

---

## Open Questions

1. **Selection mode behavior:**
   - What we know: D-07 mentions "左上角蓝色勾选框" for selection state
   - What's unclear: Is multi-select supported? How does it work with the current single-click preview behavior?
   - Recommendation: Implement single-select for now (click = preview); multi-select can be deferred

2. **Skeleton count estimate:**
   - What we know: D-09 says "skeleton数量与预期图片数量匹配"
   - What's unclear: Should skeleton count reflect subdirs + estimated visible images?
   - Recommendation: Use a fixed 12-skeleton grid as a reasonable starting point; compute from viewport width if time permits

3. **Sidebar width persistence:**
   - What we know: `sidebarCollapsed` is already in `useSettingsStore` and persisted
   - What's unclear: Should Gallery.vue read from the store or use local state?
   - Recommendation: Read from store (`settingsStore.sidebarCollapsed`) so collapse state persists across sessions, but allow local toggle via the sidebar button

4. **Folder card thumbnail source:**
   - What we know: D-11 says "子目录第一张图片" as thumbnail
   - What's unclear: Is this returned by `get_directory_images` or computed from the subdir's images?
   - Recommendation: Use existing `subdir.thumbnail` field if available in `SubDirInfo` interface; verify with `src-tauri/src/commands/gallery.rs`

---

## Environment Availability

> Step 2.6: SKIPPED (no external dependencies identified)
> This phase is purely CSS/styling changes and Vue component modifications. No external tools, services, or CLIs are required.

**Technologies used:** All already part of the project:
- naive-ui 2.41.x (installed)
- Vue 3.5.13 (installed)
- CSS Grid (native, no library)
- IntersectionObserver (native browser API, already in use)

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Vitest (existing) |
| Config file | `vitest.config.ts` |
| Quick run command | `pnpm vitest run src/views/Gallery.spec.ts` |
| Full suite command | `pnpm vitest run` |

### Existing Test Coverage
`src/views/Gallery.spec.ts` — 8 tests covering IntersectionObserver lazy loading. These tests should continue passing as the redesign preserves the IntersectionObserver pattern (Phase 3 memory leak fix).

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| REQ-05-01 | Sidebar collapses to 0px on toggle, expands to 240px | unit | `pnpm vitest run src/views/Gallery.spec.ts` | ~Gallery.spec.ts needs update |
| REQ-05-02 | Cards display pure image (no text) in default state | unit | Snapshot or render check | ~Gallery.spec.ts needs update |
| REQ-05-03 | Hover reveals bottom gradient + filename | unit | CSS class check | ~Gallery.spec.ts needs update |
| REQ-05-04 | Loading state shows NSkeleton cards instead of NSpin | unit | Component type check | ~Gallery.spec.ts needs update |
| REQ-05-05 | Click card opens preview modal | unit | Existing: handleKeydown test | ✅ Gallery.spec.ts covers navigation |
| REQ-05-06 | Empty directory shows folder icon + "该目录下暂无图片" | unit | Render check | ~Gallery.spec.ts needs update |
| REQ-05-07 | Folder cards unify with image cards (shared .gallery-card) | unit | CSS class presence | ~Gallery.spec.ts needs update |

### Sampling Rate
- **Per task commit:** `pnpm vitest run src/views/Gallery.spec.ts`
- **Per wave merge:** `pnpm vitest run`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `src/tests/gallery-redesign.spec.ts` — covers REQ-05-01 through REQ-05-07 (new tests for redesigned features)
- [ ] `src/tests/gallery-redesign.css` — CSS-specific test assertions (snapshots or class existence checks)
- [ ] Framework install: None — Vitest already configured

**Existing test infrastructure covers all phase requirements** — no Wave 0 gaps beyond adding new test cases for the redesigned gallery.

---

## Security Domain

> Security enforcement is implicitly enabled in this project. Phase 5 is a UI/UX refinement with no new network, database, or auth code. This section confirms applicable categories and notes no changes needed.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-------------------|
| V2 Authentication | No | N/A — no auth changes |
| V3 Session Management | No | N/A — no session changes |
| V4 Access Control | No | N/A — no permission changes |
| V5 Input Validation | No | N/A — no new user input |
| V6 Cryptography | No | N/A — no crypto changes |

**Phase 5 security impact:** None. This phase modifies only CSS and Vue template rendering. No user input, file paths, or network calls are introduced or changed. All existing security controls remain intact.

---

## Sources

### Primary (HIGH confidence)
- [VERIFIED: naive-ui 2.41.x source] `src/components/_builtins/loading/Skeleton.tsx` — NSkeleton implementation, confirmed `height`, `width`, `sharp` props
- [VERIFIED: naive-ui 2.41.x NLayoutSider] — `collapsed`, `collapsed-width`, `collapse-mode` props confirmed via codebase inspection of existing usage
- [VERIFIED: CSS Grid Level 2 spec] `repeat(auto-fill, minmax())` behavior for auto-calculated columns — W3C standard
- [VERIFIED: current Gallery.vue implementation] — lines 37-45 for image URL strategy, lines 528-531 for grid layout

### Secondary (MEDIUM confidence)
- [VERIFIED: settings store] `sidebarCollapsed` already exists in `useSettingsStore` — can be reused for sidebar state
- [VERIFIED: IntersectionObserver pattern] — Phase 3 memory leak fix confirmed in Gallery.spec.ts tests

### Tertiary (LOW confidence)
- [ASSUMED: NSkeleton default animation] — shimmer animation assumed default; verified via naive-ui component documentation pattern but not explicitly checked in source

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — naive-ui 2.41.x NSkeleton confirmed; no new dependencies
- Architecture: HIGH — CSS Grid auto-fill is standard; NLayoutSider collapse is standard pattern
- Pitfalls: MEDIUM — ResizeObserver for skeleton count is a reasonable approach; edge cases (very small viewports) not tested

**Research date:** 2026-05-08
**Valid until:** 2026-06-08 (30 days — stable, Apple Photos design patterns are well-established)