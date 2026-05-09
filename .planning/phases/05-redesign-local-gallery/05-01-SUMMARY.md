---
phase: "05-redesign-local-gallery"
plan: "01"
status: complete
started: "2026-05-09T23:40:00Z"
completed: "2026-05-10T00:00:00Z"
executor: gsd-executor
wave: 1
tasks_total: 3
tasks_completed: 3
---

## Plan Summary

**05-01: Apple Photos Gallery Redesign**

Extracted Gallery.vue into three focused components with Apple Photos-inspired aesthetic.

## What Was Built

### GallerySidebar.vue (119 lines)
- Collapsible 240px NLayoutSider, default expanded
- Toggle collapses to 0px via `settingsStore.sidebarCollapsed`
- Tree navigation with folder icons + image count badges
- NSkeleton loading state (D-09 compliant)

### GalleryCards.vue (249 lines)
- Apple Photos card grid: `repeat(auto-fill, minmax(160px, 1fr))`, gap: 4px
- Unified `.gallery-card` base (D-11): shared by both image and folder cards
- Hover: bottom gradient `::after` + filename reveal (D-07), no action buttons
- Default: pure image, no text labels (D-06)
- NSkeleton grid with ResizeObserver (D-09)
- Empty state: centered folder icon + "该目录下暂无图片" (D-10)

### Gallery.vue (372 lines — down from 672)
- Imports GallerySidebar + GalleryCards
- getImageSrc: convertFileSrc primary, base64 fallback on error (D-05)
- Preview modal preserved with ArrowLeft/Right/Escape keyboard nav (D-08)
- IntersectionObserver pattern (Phase 3 memory leak fix) intact

## Key Files Created/Modified

| File | Lines | Purpose |
|------|-------|---------|
| src/views/GallerySidebar.vue | 119 | Collapsible sidebar with tree nav |
| src/views/GalleryCards.vue | 249 | Apple Photos card grid |
| src/views/Gallery.vue | 372 | Main orchestrator with preview modal |

## Verification Results

- `pnpm vitest run src/views/Gallery.spec.ts`: **8/8 passed**
- `pnpm vitest run` (full suite): **118/118 passed**
- `grep "image-card\|image-overlay\|image-actions\|folder-card\|folder-preview\|folder-info" src/views/Gallery.vue`: **0 matches** (old CSS removed)
- `grep "n-spin.*loadingImages" src/views/Gallery.vue`: **0 matches** (NSpin replaced)

## Decisions Implemented

| ID | Description | Task |
|----|-------------|------|
| D-01 | Dual-panel, sidebar 240px | Task 1 |
| D-02 | Sidebar default expanded, toggle collapse | Task 1 |
| D-03 | Apple Photos style uniform cards + grid | Task 2 |
| D-04 | White #fff, border-radius 4px, hover box-shadow | Task 2 |
| D-05 | convertFileSrc primary, base64 fallback | Task 1 |
| D-06 | Default = pure image, no text labels | Task 2 |
| D-07 | Hover = gradient + filename only, no action buttons | Task 2 |
| D-08 | Click card → NModal preview ArrowLeft/Right | Task 3 |
| D-09 | Loading = NSkeleton cards (no NSpin) | Tasks 1,2,3 |
| D-10 | Empty = centered folder icon + "该目录下暂无图片" | Task 2 |
| D-11 | Folder cards unified with image cards | Task 2 |

## Security

- T-05-01 (info disclosure — filename on hover): **Accept** — filenames are local paths
- T-05-02 (img src injection): **Mitigated** — paths from Rust backend (Phase 3 protection)
- T-05-03 (DoS via ResizeObserver): **Mitigated** — standard Vue unmount disconnect

## Self-Check

- [x] All 3 tasks executed
- [x] Gallery.vue refactored to 372 lines (down from 672)
- [x] GallerySidebar.vue at 119 lines with collapsible 240px sidebar
- [x] GalleryCards.vue at 249 lines with Apple Photos card grid
- [x] All 11 decisions (D-01 through D-11) implemented
- [x] All 8 IntersectionObserver tests pass
- [x] No action buttons on card hover (D-07)
- [x] Skeleton loading replaces NSpin (D-09)
- [x] Empty state shows centered folder icon + "该目录下暂无图片" (D-10)
- [x] getImageSrc uses convertFileSrc primary (D-05)