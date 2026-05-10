---
phase: "6"
plan: "02"
subsystem: gallery-view
tags: [breadcrumb, navigation, scroll, naive-ui]
dependency_graph:
  requires:
    - "06-01"
  provides:
    - "NBreadcrumb component in Gallery.vue"
    - "breadcrumbSegments computed"
    - "handleBreadcrumbClick navigation handler"
    - "scrollToFirstCard scroll orchestration"
  affects:
    - "src/views/Gallery.vue"
tech_stack:
  added:
    - "@naive-ui NBreadcrumb, NBreadcrumbItem"
  patterns:
    - "Async navigation with nextTick scroll orchestration"
    - "Path resolution via string manipulation relative to downloadPath root"
key_files:
  created: []
  modified:
    - "src/views/Gallery.vue"
decisions:
  - id: "D-06-02-01"
    decision: "Strip downloadPath from selectedKey, split on '/', yield folder-name segments"
    rationale: "Allows NBreadcrumb to show hierarchical path without full path string"
  - id: "D-06-02-02"
    decision: "Only ancestor segments are clickable (i < length - 1); current folder is non-clickable"
    rationale: "Current segment is already active; prevents no-op enterSubdir calls"
  - id: "D-06-02-03"
    decision: "scrollToFirstCard uses viewport visibility check — skips scroll if card already visible"
    rationale: "avoids disruptive scroll when user is already at the top of the folder content"
  - id: "D-06-02-04"
    decision: "enterSubdir made async, awaits loadImagesForDirectory + nextTick before scrollToFirstCard"
    rationale: "ensures DOM is updated before querying .content-grid for first card"
metrics:
  duration: "~18 minutes"
  completed: "2026-05-10"
---

# Phase 06 Plan 02: Breadcrumb Navigation Summary

Hierarchical NBreadcrumb replacing the flat `.path-bar` in Gallery.vue, with full path resolution from `downloadPath` root, click-to-navigate, and smooth scroll to the first image card on folder entry.

## What Was Built

- **NBreadcrumb component** — replaces flat path display with naive-ui `n-breadcrumb` / `n-breadcrumb-item` loop over `breadcrumbSegments`, folder icon (#f0a020) per segment
- **breadcrumbSegments computed** — strips `downloadPath` from `selectedKey`, normalizes backslashes, splits into individual folder-name segments
- **handleBreadcrumbClick(index)** — reconstructs target path from `downloadPath + prefix`, calls `enterSubdir` with a synthetic `SubDirInfo`
- **scrollToFirstCard()** — queries `.content-grid [data-image-path]`, checks if first card is already in viewport, smooth-scrolls only when needed
- **enterSubdir refactored async** — awaits `loadImagesForDirectory`, `nextTick`, then calls `scrollToFirstCard`
- **goUp refactored** — uses async `enterSubdir` pattern (same scroll orchestration)
- **Dead code removed** — `openCurrentFolder` (no longer called from template) and `currentPath` (replaced by `breadcrumbSegments`)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Removed unused `openCurrentFolder`**
- **Found during:** TypeScript type check
- **Issue:** `TS6133: 'openCurrentFolder' is declared but its value is never read` — function was only called by the now-replaced `.path-bar` div
- **Fix:** Removed `openCurrentFolder` function entirely
- **Files modified:** `src/views/Gallery.vue`
- **Commit:** `62012a8`

**2. [Rule 3 - Blocking] Removed unused `currentPath`**
- **Found during:** TypeScript type check (second pass)
- **Issue:** `TS6133: 'currentPath' is declared but its value is never read` — `currentPath` computed was only used by the removed `.path-bar` template
- **Fix:** Removed `currentPath` computed; `breadcrumbSegments` serves the same navigation purpose
- **Files modified:** `src/views/Gallery.vue`
- **Commit:** `62012a8`

**3. [Pre-existing] GalleryCards.vue MasonryWall import warning**
- **Issue:** `TS2613: Module has no default export` in `GalleryCards.vue` — a pre-existing type error unrelated to this plan's changes
- **Status:** Not addressed — originates from `@yeger/vue-masonry-wall` package types (introduced in plan 06-01)
- **Commit:** N/A — pre-existing issue

## Verification

| Check | Result |
|-------|--------|
| `pnpm vitest run src/views/Gallery.spec.ts` | 8/8 passed |
| `pnpm typecheck` | 1 pre-existing error (MasonryWall default export, not from this plan) |
| NBreadcrumb imported from naive-ui | Pass |
| `n-breadcrumb-item` clickable gating (last item non-clickable) | Pass |
| `breadcrumbSegments.length > 0` gate on breadcrumb bar | Pass |
| `.breadcrumb-bar` CSS replaces `.path-bar` | Pass |
| `.path-bar`, `.path-text` styles removed | Pass |
| `scrollToFirstCard` viewport check before scroll | Pass |

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: pre-existing | src/views/GalleryCards.vue:7 | MasonryWall default export type issue — pre-existing, not introduced by this plan |

## Requirements Covered

| Requirement | Status |
|-------------|--------|
| REQ-06-Breadcrumb — NBreadcrumb shows full path hierarchy from downloadPath root | Complete |
| REQ-06-Navigate — Clicking ancestor segment navigates to that folder | Complete |
| REQ-06-ScrollFolderSwitch — Entering any folder scrolls to first image card | Complete |
