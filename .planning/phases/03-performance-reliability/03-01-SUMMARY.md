---
phase: "03"
plan: "01"
subsystem: frontend
tags: [lazy-loading, IntersectionObserver, memory-leak, gallery]
dependency_graph:
  requires: []
  provides:
    - Gallery.vue IntersectionObserver lazy loading
  affects:
    - src/views/Gallery.vue
    - src/views/Gallery.spec.ts
tech_stack:
  added: []
  patterns:
    - IntersectionObserver for viewport-aware lazy loading
    - LRU cache for base64 image memory management
    - onUnmounted cleanup for observer lifecycle
key_files:
  created:
    - src/views/Gallery.spec.ts
  modified:
    - src/views/Gallery.vue
decisions:
  - "IntersectionObserver with rootMargin 200px for pre-loading"
  - "loadVisibleImages observes all [data-image-path] cards via querySelectorAll"
  - "observerRef.disconnect() called in onUnmounted to prevent memory leaks"
  - "getImageSrc unchanged — LRU cache still checked first, convertFileSrc fallback preserved"
metrics:
  duration: "~30 min"
  completed_date: "2026-04-17"
  tests_added: 8
  tests_total: 118
  test_files_passed: 6
---

# Phase 03 Plan 01: IntersectionObserver Lazy Loading Summary

## One-liner

Replaced unlimited `preloadImages` in Gallery.vue with IntersectionObserver-based lazy loading that only converts visible images to base64 and disconnects the observer on component unmount.

## What Was Built

**Memory leak fix in Gallery.vue** using IntersectionObserver-based lazy loading. The previous `preloadImages` function iterated ALL images in a directory and converted each to base64, filling the LRU cache (max 100 entries) with the first 100 images regardless of viewport position.

**New approach:**
- `IntersectionObserver` watches all `[data-image-path]` image cards
- `observeCallback` fires only for visible images (`isIntersecting: true`)
- `loadImageBase64` converts only visible images to base64, caching in LRU
- `observerRef.disconnect()` called in `onUnmounted` to prevent memory leaks
- `rootMargin: '200px'` pre-loads images 200px before they enter viewport

## Key Changes

### src/views/Gallery.vue
- Added `observerRef` (IntersectionObserver instance)
- Added `observeCallback` (loads base64 only for `isIntersecting: true`)
- Added `loadImageBase64` (async LRU-cached base64 conversion)
- Added `loadVisibleImages` (observes all `[data-image-path]` cards)
- `onMounted`: creates IntersectionObserver with `rootMargin: '200px', threshold: 0.01`
- `onUnmounted`: disconnects observer (prevents memory leak)
- `refresh()`: disconnects observer before clearing state
- `data-image-path` attribute added to image/folder cards
- `defineExpose({ loadVisibleImages })` for testability

### src/views/Gallery.spec.ts
8 tests covering:
1. IntersectionObserver created with correct options (`rootMargin: '200px'`, `threshold: 0.01`)
2. `loadVisibleImages` calls `observer.observe` on all image cards
3. `isIntersecting: true` fires `loadImageBase64` via `invoke('get_local_image_base64')`
4. `isIntersecting: false` does NOT trigger loading
5. `observer.disconnect()` called on component unmount
6. Duplicate viewport entries do not re-load (LRU cache dedup)
7. `refresh()` disconnects observer before clearing state
8. LRU cache eviction behavior (max 3 entries tested)

### src/tests/setup.ts
- Resolved `vi.fn()` hoisting issue by moving all spies to module-level declarations
- Changed `ResizeObserver` mock from `vi.fn().mockImplementation()` to `class FakeResizeObserver` (required for vueuc/naive-ui compatibility)
- Fixed `listenMock` declaration order (must precede `vi.mock('@tauri-apps/api/event')`)

## Verification

```
pnpm vitest run src/views/Gallery.spec.ts
# 8 tests passed (all IntersectionObserver scenarios + LRU cache)

pnpm vitest run
# 118 tests passed across 6 test files
```

## Threat Resolution

| Threat | Resolution |
|--------|------------|
| T-03-01: DoS via unbounded base64 preloading | Fixed: Only visible images (within 200px viewport margin) are converted to base64. LRU cache caps at 100 entries. Observer disconnects on unmount. |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Test mocking hoisting conflict**
- **Found during:** Running tests
- **Issue:** `vi.mock` factories cannot reference module-level `vi.fn()` variables (Vitest hoists `vi.mock` before module-level code). Also, `vi.fn().mockImplementation()` returned a non-constructor for `ResizeObserver`, breaking vueuc/naive-ui.
- **Fix:** Used `vi.hoisted()` for all spy declarations including `fakeIntersectionObserver`. Changed `ResizeObserver` mock from `vi.fn()` to `class FakeResizeObserver` syntax for constructor compatibility.
- **Files modified:** `src/tests/setup.ts`, `src/views/Gallery.spec.ts`
- **Commit:** [fixup for 8f17980]

**2. [Minor] Test file naming**
- **Plan specified:** `src/views/Gallery.component.spec.ts`
- **Actual:** `src/views/Gallery.spec.ts` (matches existing project convention of `*.spec.ts`)
- **Decision:** Followed existing project convention

### Known Stubs

None.

## Threat Flags

None introduced by this plan.

## Self-Check

- [x] All 8 tests pass
- [x] Full suite: 118 tests across 6 files
- [x] `IntersectionObserver` in Gallery.vue (line 319)
- [x] `onUnmounted` cleanup (line 326)
- [x] `data-image-path` on image cards (lines 374, 397)
- [x] `loadVisibleImages` function (line 73)
- [x] `observerRef.disconnect()` in `refresh()` (line 309)
- [x] No stubs in implementation
- [x] No hardcoded secrets or security issues
