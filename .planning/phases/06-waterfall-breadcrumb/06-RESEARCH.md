# Phase 06: 瀑布流布局 + 面包屑导航 - Research

**Researched:** 2026-05-10
**Domain:** Vue 3 masonry layout + breadcrumb navigation
**Confidence:** HIGH

## Summary

Phase 06 refactors the image grid from CSS Grid (`auto-fill, minmax(160px, 1fr)`) to true JS-driven masonry using `@yeger/vue-masonry-wall`, and replaces the flat path bar with a hierarchical `NBreadcrumb` component derived from image file paths relative to `downloadPath`. Clicking any breadcrumb segment navigates to that folder and scrolls to the first image card. The core integration points are in `GalleryCards.vue` (masonry container, image card rendering) and `Gallery.vue` (breadcrumb, folder navigation, scroll orchestration).

**Primary recommendation:** Install `@yeger/vue-masonry-wall` v6.1.1, replace `.content-grid` CSS Grid with `<MasonryWall>`, compute breadcrumb segments from the current folder path relative to `downloadPath`, and use `nextTick` + `scrollIntoView` for smooth scroll-to-first-image on breadcrumb clicks.

## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Implementation = JS masonry library (`@yeger/vue-masonry-wall`), not CSS `column-count`
- **D-02:** Breadcrumb data = from image file path resolution (relative to `downloadPath`), not parent-child folder tree
- **D-03:** Breadcrumb click = navigate to folder + scroll to first image visible in viewport
- **D-04:** Folder switch scroll = `scrollIntoView({ behavior: 'smooth', block: 'start' })`

### Claude's Discretion

- Masonry `column-width` value (160px retained from existing grid)
- Whether to show a folder list above the masonry grid (keep existing `folder-list` above grid)
- Specific naive-ui `NBreadcrumb` theming / styling

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.

---

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| REQ-06-Masonry | Replace CSS Grid with @yeger/vue-masonry-wall | Section "Library Deep Dive: vue-masonry-wall" |
| REQ-06-Breadcrumb | Replace path bar with hierarchical NBreadcrumb | Section "Breadcrumb Path Resolution" |
| REQ-06-Navigate | Click breadcrumb segment -> navigate + scroll to first image | Section "Scroll Behavior on Navigation" |
| REQ-06-ScrollFolderSwitch | Smooth scroll to first image card on folder switch | Section "Scroll Behavior on Navigation" |
| REQ-06-Responsive | Keep UI clean and responsive | Section "Responsive Behavior" |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `@yeger/vue-masonry-wall` | 6.1.1 | True masonry waterfall layout | Pure Vue 3, no jQuery, TypeScript support, SSR-safe, `column-width` + `gap` props, responsive |
| `naive-ui` | 2.41.0 | `NBreadcrumb` + `NBreadcrumbItem` | Already used in project, consistent UI language |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `IntersectionObserver` (native) | — | Preserve Phase 3 lazy loading | Every image card observed, LRU cache on intersect |
| `convertFileSrc` | — | Primary image URL (Tauri asset) | Unchanged from Phase 5 |
| `ResizeObserver` (native) | — | Responsive column count | Replaces current `updateSkeletonCount` in MasonryWall context |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `@yeger/vue-masonry-wall` | CSS `column-count` | Rejected — column-first ordering breaks row priority, scrollbar jitter on height changes |
| `@yeger/vue-masonry-wall` | `vue-masonry` | Rejected — older library, Vue 2 legacy, less maintained |

**Installation:**
```bash
pnpm add @yeger/vue-masonry-wall
```

---

## Architecture Patterns

### Recommended Project Structure

```
src/views/
├── Gallery.vue          # Breadcrumb logic, folder nav, scroll orchestration
├── GalleryCards.vue     # Masonry container, image/folder card rendering
└── Gallery.spec.ts      # Updated tests for masonry + breadcrumb
```

### Pattern 1: Masonry Container

**What:** Replace `.content-grid` (CSS Grid) with `<MasonryWall>` from `@yeger/vue-masonry-wall`.
**When to use:** Image grid with variable-height cards (portrait/landscape/square mixed).
**Key prop mapping:**
- `items` — the array of images (or combined images+subdirs)
- `column-width` — `160` (retained from existing grid card size)
- `gap` — `4` (matches existing `gap: 4px`)
- `min-columns` — `1` (prevents single-column on narrow viewports)

**Example structure:**
```vue
<MasonryWall :items="displayItems" :column-width="160" :gap="4" :min-columns="1">
  <template #default="{ item, index }">
    <ImageCard :item="item" :index="index" @click="..." />
  </template>
</MasonryWall>
```

### Pattern 2: Breadcrumb Path Resolution

**What:** Compute breadcrumb segments by stripping `downloadPath` prefix from `selectedKey`, then splitting remaining path.
**When to use:** Display hierarchical folder context in breadcrumb UI.
**Algorithm:**
```
input:  selectedKey = "C:/Users/Downloads/Gelbooru/ArtistA"
        downloadPath = "C:/Users/Downloads/Gelbooru"
output: segments = ["Gelbooru", "ArtistA"]
```

```typescript
// breadcrumbSegments computed in Gallery.vue
const breadcrumbSegments = computed(() => {
  if (!selectedKey.value || !settingsStore.downloadPath) return [];
  const relative = selectedKey.value
    .replace(/\\/g, '/')
    .replace(settingsStore.downloadPath.replace(/\\/g, '/'), '')
    .replace(/^\/+|\/+$/g, '');
  if (!relative) return [];
  return relative.split('/');
});
```

### Pattern 3: Click + Scroll Orchestration

**What:** On breadcrumb segment click — (1) call `enterSubdir`, (2) wait for `nextTick`, (3) find first card in that folder, (4) call `scrollIntoView`.
**When to use:** Breadcrumb navigation and folder switching.

```typescript
// In Gallery.vue — scroll to first image after entering a directory
async function enterSubdir(subdir: SubDirInfo) {
  selectedKey.value = subdir.path;
  await loadImagesForDirectory(subdir.path);
  await nextTick();
  scrollToFirstCard();
}

// Check if already visible before scrolling (D-03: no scroll if folder in viewport)
function scrollToFirstCard() {
  const firstCard = document.querySelector('.content-grid [data-image-path]') as HTMLElement;
  if (!firstCard) return;
  // Check bounding rect — only scroll if card is not in current viewport
  const rect = firstCard.getBoundingClientRect();
  const inViewport = rect.top >= 0 && rect.bottom <= window.innerHeight;
  if (!inViewport) {
    firstCard.scrollIntoView({ behavior: 'smooth', block: 'start' });
  }
}
```

### Pattern 4: Combined Display Items

**What:** Masonry wall renders folder cards + image cards as a unified list.
**When to use:** Masonry grid showing both subdirs and images in the same layout.

```typescript
// Combine subdirs and images into a single items array for MasonryWall
const displayItems = computed(() => {
  const folderItems = subdirs.value.map((s) => ({ ...s, _type: 'folder' as const }));
  const imageItems = images.value.map((i) => ({ ...i, _type: 'image' as const }));
  return [...folderItems, ...imageItems];
});
```

### Anti-Patterns to Avoid

- **CSS `column-count` for masonry:** Causes column-first ordering (last image in row appears first in DOM), scrollbar jumps when column heights change. Use JS masonry instead.
- **`scrollIntoView` without `nextTick`:** The masonry layout re-renders after `loadImagesForDirectory`; querying DOM before Vue's next render cycle finds stale elements.
- **Breadcrumb from folder tree API:** D-02 explicitly uses image path resolution — do not refetch `get_directory_tree` for breadcrumb data since image paths are already available.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Masonry layout | Custom JS positioning | `@yeger/vue-masonry-wall` | Edge cases (resize, dynamic items, SSR) already handled |
| Breadcrumb separator styling | Custom CSS separators | `NBreadcrumbItem` with `NBreadcrumb` | Consistent with naive-ui theme, `separator` prop |
| Scroll position math | Manual scroll calculations | `scrollIntoView` | Native browser API, handles all edge cases |

---

## Common Pitfalls

### Pitfall 1: Masonry `items` reference mutation

**What goes wrong:** Changing `images.value = newImages` triggers full re-render; MasonryWall re-calculates all positions, causing visible layout flicker.
**Why it happens:** Vue's reactivity system replaces the array; MasonryWall sees `items` change and recalculates layout.
**How to avoid:** Keep `items` array stable between navigation — use the same array and replace contents in place when possible. For navigation between folders, a full reset is unavoidable but should be preceded by a skeleton state.

### Pitfall 2: Image card click target lost in MasonryWall scoped slot

**What goes wrong:** Clicking a card inside the MasonryWall slot does not trigger the card's `@click` handler.
**Why it happens:** MasonryWall wraps the slot content in its own container `div`; event propagation depends on whether the slot root element handles clicks.
**How to avoid:** Ensure each card has a `@click` handler on its root element (the `.gallery-card` div). Do not rely on event delegation from a parent.

### Pitfall 3: IntersectionObserver observe after masonry render

**What goes wrong:** IntersectionObserver re-connects but observes old card elements that were removed from DOM during masonry re-layout.
**Why it happens:** `loadVisibleImages()` queries `.content-grid [data-image-path]` after `loadImagesForDirectory`, but MasonryWall's render cycle has not completed yet.
**How to avoid:** Call `loadVisibleImages()` after `await nextTick()` — MasonryWall uses `nextTick` internally, so waiting one more tick ensures all DOM elements are in place.

### Pitfall 4: Breadcrumb segments empty on root folder

**What goes wrong:** When `selectedKey === downloadPath`, relative path is empty, breadcrumb shows nothing.
**Why it happens:** Stripping `downloadPath` from `selectedKey` leaves nothing.
**How to avoid:** Show no breadcrumb (or a single "根目录" segment) when `selectedKey` equals `downloadPath`. Handle `selectedKey === null` (initial state) as no breadcrumb.

### Pitfall 5: Scroll triggered for already-visible folder

**What goes wrong:** Clicking a breadcrumb segment for a folder whose images are already in the current viewport still triggers `scrollIntoView`, causing visible jump.
**Why it happens:** `enterSubdir` is called even when the target folder is the current folder.
**How to avoid:** In `handleBreadcrumbClick`, check `if (targetPath === selectedKey.value) return`. Additionally, D-04 suggests checking viewport visibility before scrolling.

---

## Code Examples

### Breadcrumb Component Integration

Source: naive-ui docs — `NBreadcrumb` + `NBreadcrumbItem` usage in Vue 3 Composition API.

```vue
<!-- Replace .path-bar div in Gallery.vue -->
<template v-if="breadcrumbSegments.length > 0">
  <n-breadcrumb style="margin-bottom: 8px">
    <n-breadcrumb-item
      v-for="(segment, i) in breadcrumbSegments"
      :key="i"
      :clickable="i < breadcrumbSegments.length - 1"
      @click="i < breadcrumbSegments.length - 1 && handleBreadcrumbClick(i)"
    >
      <n-icon :size="14" style="margin-right: 4px">
        <FolderOpenOutline />
      </n-icon>
      {{ segment }}
    </n-breadcrumb-item>
  </n-breadcrumb>
</template>
```

### MasonryWall with Combined Items

Source: `@yeger/vue-masonry-wall` v6 — usage pattern.

```vue
<script setup lang="ts">
import { computed } from 'vue';
import MasonryWall from '@yeger/vue-masonry-wall';
import type { ImageInfo, SubDirInfo } from './GalleryCards.vue';

// Combine folder + image items for masonry
const displayItems = computed<(ImageInfo & { _type: 'image' }) | (SubDirInfo & { _type: 'folder' })[]>(() => [
  ...subdirs.value.map((s) => ({ ...s, _type: 'folder' as const })),
  ...images.value.map((i) => ({ ...i, _type: 'image' as const })),
]);
</script>

<template>
  <MasonryWall
    v-if="displayItems.length > 0"
    :items="displayItems"
    :column-width="160"
    :gap="4"
    :min-columns="1"
    class="content-grid"
  >
    <template #default="{ item, index }">
      <div
        class="gallery-card"
        :class="{ 'folder-card': item._type === 'folder' }"
        :data-image-path="item._type === 'image' ? item.path : (item.thumbnail ?? '')"
        @click="item._type === 'folder' ? emit('enter-subdir', item) : emit('open-preview', index - subdirs.length)"
      >
        <!-- card content — same as current GalleryCards.vue -->
      </div>
    </template>
  </MasonryWall>
</template>
```

### Path Resolution for Breadcrumb

```typescript
// Gallery.vue — compute breadcrumb segments from selectedKey
function getBreadcrumbSegments(path: string, root: string): string[] {
  const normalizedPath = path.replace(/\\/g, '/');
  const normalizedRoot = root.replace(/\\/g, '/');
  if (!normalizedPath.startsWith(normalizedRoot)) return [];
  const relative = normalizedPath.slice(normalizedRoot.length).replace(/^\/+|\/+$/g, '');
  if (!relative) return [];
  return relative.split('/');
}

const breadcrumbSegments = computed(() =>
  getBreadcrumbSegments(selectedKey.value ?? '', settingsStore.downloadPath)
);

function handleBreadcrumbClick(index: number) {
  // Rebuild path up to (and including) this segment
  const prefix = breadcrumbSegments.value.slice(0, index + 1).join('/');
  const targetPath = `${settingsStore.downloadPath.replace(/\\/g, '/')}/${prefix}`;
  const targetSubdir: SubDirInfo = { path: targetPath, name: breadcrumbSegments.value[index], imageCount: 0 };
  enterSubdir(targetSubdir);
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| CSS Grid `auto-fill, minmax(160px, 1fr)` | JS Masonry via `@yeger/vue-masonry-wall` | Phase 06 | True variable-height masonry, no row-height gaps |
| Flat path bar `.path-bar` | Hierarchical `NBreadcrumb` | Phase 06 | Clickable ancestor folders, better UX |
| Folder list horizontal pills above grid | Keep (folder list) + add breadcrumb above it | Phase 06 | Folder navigation remains accessible |

**Deprecated/outdated:**
- CSS `column-count` masonry: Deprecated by D-01 decision — not a real masonry, causes ordering issues.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `@yeger/vue-masonry-wall` v6.1.1 is compatible with Vue 3.5 and Tauri environment | Standard Stack | LOW — v6 is Vue 3 specific, tested with Vue 3.5+ per npm description |
| A2 | `MasonryWall` can accept a mixed-type items array (folder + image combined) | Code Examples | LOW — component accepts `any[]` for `items` prop |
| A3 | `NBreadcrumb` supports clickable/non-clickable items via `clickable` prop | Common Pitfalls | LOW — naive-ui NBreadcrumbItem has `clickable` prop |
| A4 | `scrollIntoView` behavior `smooth` works in Tauri WebView (Chromium-based) | Scroll Behavior | LOW — Tauri 2.x uses WebView2 on Windows, supports smooth scroll |

**If this table is empty:** All claims in this research were verified or cited — no user confirmation needed.

---

## Open Questions

1. **Should the folder list (`.folder-list`) remain above the masonry grid after Phase 06?**
   - What we know: Current design has folder pills above the image grid. Breadcrumb replaces the path bar, not necessarily the folder list.
   - What's unclear: Whether keeping the folder list + adding breadcrumb creates redundancy (both show folder context).
   - Recommendation: Keep folder list (user can still click to navigate subdirs horizontally). Breadcrumb sits above folder list. If too crowded, fold the breadcrumb into a single row above everything.

2. **Should the ".." up navigation button be replaced with breadcrumb home segment?**
   - What we know: Current design has a `..` folder-item that calls `goUp()`. Breadcrumb shows full path hierarchy.
   - What's unclear: Whether clicking breadcrumb root should be equivalent to ".." or go all the way to downloadPath root.
   - Recommendation: Click on first breadcrumb segment (downloadPath root name) goes to root folder. Keep ".." for consistency with existing navigation.

---

## Environment Availability

Step 2.6: SKIPPED (no external dependencies beyond project code — this is a pure code/configuration change).

The phase modifies:
- `GalleryCards.vue` — replace CSS Grid with MasonryWall
- `Gallery.vue` — add breadcrumb, folder nav refactor
- New package: `@yeger/vue-masonry-wall` (install via pnpm)

No external services, CLI tools, or runtime dependencies required beyond `pnpm add`.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Vitest (existing) + @vue/test-utils |
| Config file | `vitest.config.ts` (not found — uses `pnpm vitest` default config from package.json) |
| Quick run command | `pnpm vitest run src/views/Gallery.spec.ts` |
| Full suite command | `pnpm test` |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| REQ-06-Masonry | MasonryWall renders with correct column-width and gap | unit | `pnpm vitest run src/views/Gallery.spec.ts` | partially (Gallery.spec.ts exists, needs new test cases) |
| REQ-06-Breadcrumb | Breadcrumb segments computed correctly from path | unit | `pnpm vitest run src/views/Gallery.spec.ts` | no |
| REQ-06-Navigate | Click breadcrumb segment navigates + scrolls | integration | manual | no |
| REQ-06-ScrollFolderSwitch | enterSubdir scrolls to first card | unit | `pnpm vitest run src/views/Gallery.spec.ts` | no |
| REQ-06-Responsive | Masonry reflows on window resize | unit | `pnpm vitest run src/views/Gallery.spec.ts` | no |

### Sampling Rate

- **Per task commit:** `pnpm vitest run src/views/Gallery.spec.ts`
- **Per wave merge:** `pnpm test`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `src/views/Gallery.spec.ts` — add test cases for breadcrumb path resolution and masonry rendering
- [ ] Framework install: `@yeger/vue-masonry-wall` needs to be added to `package.json` (pnpm add) — planner includes this as first task

*(No test framework gaps — existing Vitest + @vue/test-utils covers all needs)*

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | yes | Path-based operations use `selectedKey` from Tauri IPC response (not user input); path resolution uses safe string ops |
| V4 Access Control | no | Read-only gallery browsing, no authorization changes |

### Known Threat Patterns for Vue 3 / Tauri

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal via selectedKey | Information Disclosure | `selectedKey` comes from `get_directory_images` IPC response — server-side, already path-cleaned |
| XSS via breadcrumb segment injection | Spoofing | Breadcrumb segments are path names from file system, not user input; rendered as text, not HTML |

---

## Sources

### Primary (HIGH confidence)
- `@yeger/vue-masonry-wall` npm registry — v6.1.1, Vue 3 only, TypeScript support, `pnpm show @yeger/vue-masonry-wall version`
- naive-ui `NBreadcrumb` + `NBreadcrumbItem` — project already uses naive-ui 2.41.0, component API stable

### Secondary (MEDIUM confidence)
- MasonryWall v6 usage patterns — based on npm description and standard Vue 3 component patterns

### Tertiary (LOW confidence)
- Scroll behavior in Tauri WebView2 — [ASSUMED] Tauri 2.x uses WebView2 which supports smooth scroll; confirmed by project using Tauri 2.x

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — `@yeger/vue-masonry-wall` v6.1.1 verified on npm registry; naive-ui already in project
- Architecture: HIGH — masonry slot pattern, breadcrumb path resolution both straightforward
- Pitfalls: MEDIUM — edge cases (IntersectionObserver with MasonryWall, scroll optimization) need implementation verification

**Research date:** 2026-05-10
**Valid until:** 2026-06-10 (30 days — library version unlikely to change significantly)