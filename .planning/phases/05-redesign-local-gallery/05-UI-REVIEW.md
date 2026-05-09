# Phase 05 — UI Review

**Audited:** 2026-05-09
**Baseline:** Abstract 6-pillar standards (no UI-SPEC.md)
**Screenshots:** not captured (no dev server running)
**Files audited:**
- `src/views/Gallery.vue` (372 lines)
- `src/views/GalleryCards.vue` (249 lines)
- `src/views/GallerySidebar.vue` (119 lines)

---

## Pillar Scores

| Pillar | Score | Key Finding |
|--------|-------|-------------|
| 1. Copywriting | 4/4 | All strings are specific and localized; no generic placeholders |
| 2. Visuals | 3/4 | Apple Photos grid largely correct; folder cards violate D-06 by showing text by default |
| 3. Color | 4/4 | Palette is controlled; orange (#f0a020) scoped to folder icon only; no hardcoded colors on cards |
| 4. Typography | 3/4 | 5 distinct sizes (12–18px) with consistent medium/normal weights; no CSS font system |
| 5. Spacing | 2/4 | 8+ hardcoded px values without a defined scale; no CSS custom properties |
| 6. Experience Design | 3/4 | Strong state coverage; NSpin in sidebar violates D-09; tree has no error feedback |

**Overall: 19/24**

---

## Top 3 Priority Fixes

1. **Folder card text by default (D-06 violation)** — `.gallery-card.folder-card .card-filename` is always visible; move to `.gallery-card.folder-card:hover .card-filename` via CSS variant to match image cards' hover-only reveal
2. **GallerySidebar uses NSpin instead of NSkeleton (D-09 violation)** — Replace `n-spin :show="loadingTree"` with the same NSkeleton grid pattern used in GalleryCards.vue; create `.tree-skeleton-grid` CSS that mirrors `.content-grid` skeleton layout
3. **No consistent spacing scale** — Define CSS custom properties in `Gallery.vue` or a `gallery-tokens.css` partial (e.g., `--space-xs: 4px; --space-sm: 8px; --space-md: 12px; --space-lg: 16px; --space-xl: 18px`) and replace hardcoded values across all three files

---

## Detailed Findings

### Pillar 1: Copywriting (4/4)

All text strings are specific and context-appropriate:

| File | String | Assessment |
|------|--------|------------|
| Gallery.vue:246 | `本地图库` | Contextual heading — correct |
| Gallery.vue:252 | `刷新` | Precise action label — correct |
| Gallery.vue:198 | `确认删除` / `删除` / `取消` | Specific confirmation dialog — correct |
| Gallery.vue:209 | `删除成功` / `删除失败` | User-facing feedback — correct |
| GallerySidebar.vue:102 | `暂无本地图片` | Specific empty state — correct |
| GalleryCards.vue:132 | `该目录下暂无图片` | Specific empty state — correct |
| GalleryCards.vue:106 | `{{ subdir.name }} ({{ subdir.imageCount }})` | Dynamic but specific — correct |

**No generic patterns found** (no "Submit", "Click Here", "OK", "No data", "Error", "try again").

---

### Pillar 2: Visuals (3/4)

**Correct implementations:**
- Apple Photos dual-panel layout: `<n-layout has-sider>` + 240px fixed sidebar (D-01)
- Sidebar collapsible: `collapsed-width="0"`, `collapse-mode="width"`, `transition: width 0.2s` (D-02)
- Image cards: `aspect-ratio: 1`, `border-radius: 4px`, `background: #fff`, no border (D-04)
- Hover box-shadow: `0 2px 8px rgba(0,0,0,0.15)` + z-index lift on hover (D-04)
- Hover gradient via `::after` pseudo-element at bottom 50% (D-07)
- Filename overlay on hover only via `opacity: 0` / `opacity: 1` (D-07)
- Unified `.gallery-card` base shared by both image and folder cards (D-11)

**Issue found — folder cards violate D-06 (default = pure image, no text):**

```
src/views/GalleryCards.vue:106
<div class="card-filename">{{ subdir.name }} ({{ subdir.imageCount }})</div>
```

`.card-filename` defaults to `opacity: 0` and becomes visible on `.gallery-card:hover`. However, this rule applies to all `.gallery-card` children, including folder cards. The fix: add a CSS variant targeting `.gallery-card.folder-card .card-filename` that always hides it, letting hover reveal it — matching D-06 behavior.

**No focal point hierarchy issue** — cards are uniform, preview modal centers image content.

---

### Pillar 3: Color (4/4)

**Palette analysis:**
- Cards: `#fff` background (CSS variable `background: #fff`)
- Folder icon: `#f0a020` (amber — used only on FolderIcon in GallerySidebar)
- Path bar: `#f5f5f5` background, `#ebebeb` hover, `#666` text
- Empty state: `#999` icon/text
- Hover gradient: `linear-gradient(transparent, rgba(0,0,0,0.6))`
- Selection: `#007aff` noted in spec, not implemented (Phase 5 scope excludes selection mode — D-07)

**No accent color overuse.** Orange is scoped to a single icon. No hardcoded colors on cards.

---

### Pillar 4: Typography (3/4)

**Font size distribution:**
```
18px  (Gallery.vue:246 — page title, font-weight: 500)
13px  (Gallery.vue:354 — path bar text)
12px  (GalleryCards.vue:188 — card filename; GallerySidebar.vue:50 — image count badge)
14px  (GalleryCards.vue:245 — empty state text)
```

**Font weight distribution:**
- `font-weight: 500` — page title only
- `font-weight: normal` (default) — all other text

**Gap: No CSS font system.** Font sizes are hardcoded in inline styles. A token system would make the design more maintainable and consistent.

---

### Pillar 5: Spacing (2/4)

**Hardcoded pixel values found across files:**

```
GallerySidebar.vue:74  content-style="padding: 8px;"
GallerySidebar.vue:80  style="margin-bottom: 8px; width: 100%"
GallerySidebar.vue:49  padding: 2px 6px (badge)

GalleryCards.vue:141   gap: 4px; padding: 4px
GalleryCards.vue:184-186  bottom: 8px; left: 8px; right: 8px (filename)
GalleryCards.vue:55    margin: 0 (empty text)

Gallery.vue:245        style="margin-bottom: 16px"
Gallery.vue:341-342    padding: 8px 12px; margin-bottom: 12px
Gallery.vue:369        gap: 16px
```

**Values in use:** 4px, 8px, 12px, 16px, 18px — no defined scale.

**Contrast:** GalleryCards uses 4px gap (D-03: "tight Apple Photos grid") correctly; Gallery.vue uses 16px gap (standard layout spacing). These are intentional context differences, but a CSS scale would clarify intent.

**No CSS custom properties for spacing.** Adding a token file (`--space-xs: 4px; --space-sm: 8px; --space-md: 12px; --space-lg: 16px`) would make the design system maintainable.

---

### Pillar 6: Experience Design (3/4)

**Present and working:**
- Loading state: NSkeleton grid with ResizeObserver adaptive count (`GalleryCards.vue:32-60,78-82`) ✅ — D-09 met
- Empty state: Folder icon + "该目录下暂无图片" (`GalleryCards.vue:128-133`) ✅ — D-10 met
- Error handling: `handleImageError` with base64 fallback (`GalleryCards.vue:40-51`, `Gallery.vue:178-187`) ✅
- Destructive action confirmation: `dialog.warning()` before delete (`Gallery.vue:196-216`) ✅
- Keyboard navigation: ArrowLeft/Right/Escape in preview modal (`Gallery.vue:118-123,296-326`) ✅
- Hover feedback: gradient overlay + filename reveal + z-index lift ✅
- Lazy loading: IntersectionObserver preserved from Phase 3 (`Gallery.vue:39-54,66-71`) ✅

**Issues:**
- **GallerySidebar uses NSpin for loadingTree (D-09 violation):** `GallerySidebar.vue:92` — `n-spin :show="loadingTree"`. The rest of the app uses NSkeleton for loading states. Sidebar loading should use the same NSkeleton grid pattern for visual consistency and D-09 compliance.
- **Tree load error not surfaced to user:** `Gallery.vue:152` catches tree loading errors but only logs to `console.error`. No user-facing feedback (no `message.error()` or NEmpty with error context).
- **No tree skeleton:** Even without NSpin, there is no NSkeleton placeholder for the tree during `loadingTree`. The NSpin wraps the tree but should be replaced with NSkeleton cards matching the sidebar's width.

---

## Files Audited

- `src/views/Gallery.vue`
- `src/views/GalleryCards.vue`
- `src/views/GallerySidebar.vue`

Registry audit: not applicable (no shadcn/ui, no third-party component registries)