---
phase: 5
slug: redesign-local-gallery
status: planned
nyquist_compliant: true
wave_0_complete: true
created: 2026-05-08
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Vitest |
| **Config** | `vitest.config.ts` |
| **Quick Run** | `pnpm vitest run src/views/Gallery.spec.ts` |
| **Full Suite** | `pnpm vitest run` |
| **Estimated runtime** | ~15 seconds (Gallery.spec.ts), ~45 seconds (full suite) |

---

## Sampling Rate

- **After every task commit:** `pnpm vitest run src/views/Gallery.spec.ts`
- **After every wave:** `pnpm vitest run`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds (Gallery.spec.ts), 45 seconds (full suite)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File | Status |
|---------|------|------|-------------|-----------|-----------------|------|--------|
| 5.1-01 | 01 | 1 | Sidebar collapses/expands at 240px | grep | `grep "n-layout-sider" src/views/GallerySidebar.vue` | GallerySidebar.vue | planned |
| 5.1-02 | 01 | 1 | getImageSrc uses convertFileSrc primary | grep | `grep "imageBase64Cache.get" src/views/Gallery.vue` → 0 | Gallery.vue | planned |
| 5.1-03 | 01 | 1 | IntersectionObserver preserved | unit | `pnpm vitest run src/views/Gallery.spec.ts` | Gallery.spec.ts | planned |
| 5.2-01 | 01 | 1 | Grid uses `auto-fill, minmax(160px, 1fr)` gap 4px | grep | `grep "auto-fill" src/views/GalleryCards.vue` | GalleryCards.vue | planned |
| 5.2-02 | 01 | 1 | Cards white #fff, radius 4px, no border | grep | `grep "border:" src/views/GalleryCards.vue` → 0 | GalleryCards.vue | planned |
| 5.2-03 | 01 | 1 | Hover gradient via CSS ::after | grep | `grep "::after" src/views/GalleryCards.vue` | GalleryCards.vue | planned |
| 5.2-04 | 01 | 1 | Filename hidden by default, shown on hover | grep | `grep "opacity: 0" src/views/GalleryCards.vue` | GalleryCards.vue | planned |
| 5.3-01 | 01 | 1 | Preview modal preserved (ArrowLeft/Right) | unit | `pnpm vitest run src/views/Gallery.spec.ts` | Gallery.spec.ts | planned |
| 5.3-02 | 01 | 1 | NSkeleton replaces NSpin for loading | grep | `grep "n-spin.*loadingImages" src/views/Gallery.vue` → 0 | Gallery.vue | planned |
| 5.3-03 | 01 | 1 | Empty state: folder icon + "该目录下暂无图片" | grep | `grep "该目录下暂无图片" src/views/GalleryCards.vue` | GalleryCards.vue | planned |
| 5.3-04 | 01 | 1 | Old card CSS removed from Gallery.vue | grep | `grep -c "image-card\|image-overlay\|image-actions\|folder-card" src/views/Gallery.vue` → 0 | Gallery.vue | planned |

*Status: planned · ✅ green · ⬜ pending*

---

## Wave 0 Requirements

- [x] No new test framework needed — Vitest already configured
- [x] Gallery.spec.ts stubs already exist — add `n-skeleton`, `gallery-cards`, `gallery-sidebar` stubs (inlined in Plan 01 Task 3)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Sidebar collapse animation smooth | D-02 | Visual timing check | Toggle sidebar — verify 0.2s ease transition, no flicker |
| Hover gradient visible at correct position | D-07 | Visual placement | Hover card — verify gradient at bottom 50%, filename visible |
| Card aspect ratio 1:1 at various sizes | D-04 | Responsive layout | Resize window — cards maintain square shape via `aspect-ratio: 1` |
| Skeleton cards match grid layout | D-09 | Visual comparison | Open gallery with loading — skeleton grid matches card grid columns |
| Empty state centered in viewport | D-10 | Visual centering | Navigate to empty folder — icon + text centered vertically/horizontally |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: per-task grep checks + 2 automated test runs per task
- [x] Wave 0 covers all MISSING references (stubs inlined in Plan 01 Task 3)
- [x] No watch-mode flags
- [x] Feedback latency < 45s (full suite), < 15s (Gallery.spec.ts)
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** planned
