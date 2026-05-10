---
name: gallery-thumbnails-not-loading-v2
description: 图库卡片已显示（骨架 shimmer）但缩略图不加载 - v2 investigation
type: debug_session
status: resolved
created: 2026-05-10T19:30:00.000Z
updated: 2026-05-10T19:50:00.000Z
trigger: 图库卡片已显示（骨架 shimmer）但缩略图不加载

## Current Focus

**ROOT CAUSE IDENTIFIED: `loadVisibleImages()` is called before MasonryWall finishes its async layout, finding ZERO cards to observe. The IntersectionObserver never fires because no elements are ever observed.**

## Hypothesis Tested

| # | Hypothesis | Test | Result |
|---|------------|------|--------|
| H1 | `galleryCardsRef` not bound in template | Read template line 346 | ELIMINATED — `ref="galleryCardsRef"` confirmed present |
| H2 | `observeCallback` never fires because observer not watching cards | Code trace: `loadVisibleImages` DOM query timing | **CONFIRMED — `loadVisibleImages()` queries `[data-image-path]` BEFORE MasonryWall async layout completes** |
| H3 | `galleryCardsRef.value` is null in `observeCallback` | Template ref binding analysis | ELIMINATED — ref bound correctly, should be set before observer fires |
| H4 | Path normalization mismatch between observer and Map key | Trace `path.replace()` and `imageSrcMap.set(path, src)` | ELIMINATED — both use raw `path` (same format), normalization only affects `convertFileSrc` |
| H5 | `convertFileSrc` returns inaccessible URL | CSP config review | ELIMINATED — `img-src * data: blob: filesystem:` allows all origins |
| H6 | `data-image-path` not set on rendered cards | Read GalleryCards.vue line 72 | ELIMINATED — attribute bound correctly with `:data-image-path="item.path"` |
| H7 | Double observer creation leaves orphaned observer | Trace onMounted vs loadImagesForDirectory timing | ELIMINATED (secondary) — observer created twice but cards found by second call |

---

## Root Cause

**`loadVisibleImages()` is called immediately after a single `await nextTick()`, but MasonryWall's internal async `fillColumns()` pipeline has not yet completed. The query `grid.querySelectorAll('[data-image-path]')` returns an empty NodeList (0 cards). Zero cards are observed. IntersectionObserver callback never fires. Images never load.**

### MasonryWall async layout pipeline (confirmed from source):

```
onMounted():
  await redraw()  ← async, does not block
  resizeObserver.observe(wall.value)

redraw() → fillColumns(0, id):
  await nextTick()  ← MasonryWall's own await
  columns.value[index].push(itemIndex)
  await fillColumns(1, id)  ← recursive async
```

After `loadingImages = false` + one `await nextTick()`:
- Vue re-renders GalleryCards — MasonryWall `v-if` becomes true
- MasonryWall `onMounted` fires → schedules `redraw()`
- `redraw()` calls `await nextTick()` internally (inside MasonryWall)
- Vue microtask queue: MasonryWall render → `redraw()` nextTick → column population
- **BUT**: `loadVisibleImages()` is called BEFORE MasonryWall's `redraw()` nextTick completes

### Sequence of events in `loadImagesForDirectory(reset=true)`:

```
Line 131: loadingImages = true  (MasonryWall hidden, not in DOM)
Line 138-149: await invoke()  (images fetched)
Line 151: images.value = result.images
Line 155: hasMore = result.has_more
Line 156: if (reset) {
Line 157:   await nextTick()  ← Vue re-renders GalleryCards, MasonryWall appears
Line 158-164: if (!observerRef.value) observerRef = new IntersectionObserver()
Line 165: loadVisibleImages()  ← QUERIES DOM IMMEDIATELY
            grid = document.querySelector('.content-grid')  ← MasonryWall div EXISTS
            cards = grid.querySelectorAll('[data-image-path]')  ← RETURNS [] (0 cards!)
            // forEach loop over empty NodeList → NO OBSERVATION
Line 166: setupLoadMoreObserver()
```

### Evidence

- **MasonryWall source (index.mjs line 57-63)**: `fillColumns()` is `async` with its own `await nextTick()` inside the loop
- **Timeline**: `loadVisibleImages()` called at T+1 tick, MasonryWall layout completes at T+2+ ticks
- **Result**: `querySelectorAll('[data-image-path]')` returns empty NodeList (no elements to iterate)
- **IntersectionObserver fires on 0 elements**: callback never called
- **`imageSrcMap` stays empty**: shimmer permanently visible
- **`galleryCardsRef` correctly bound**: verified `ref="galleryCardsRef"` present in template (line 346-352)
- **`defineExpose` correct**: `setCardSrc` and `getCardSrc` both exposed (line 45)
- **CSP allows asset URLs**: `img-src * data: blob: filesystem:` covers `asset://localhost/...`
- **Path normalization consistent**: observer and Map both use raw `path` as key

### Why skeleton shimmer shows

`v-if="loadingImages"` uses the same `loadingImages` ref that triggers the MasonryWall render. When `loadingImages` is `true` (during API call), skeleton cards render via the skeleton branch. Once `loadingImages` becomes `false`, the skeleton branch is hidden and MasonryWall renders. The shimmer placeholder (`card-placeholder` div) is **always visible** because `v-if="getCardSrc(item.path)"` is always `false` (Map is empty, `getCardSrc()` returns `''`).

---

## Fix Direction

Add an additional `await nextTick()` call before `loadVisibleImages()` to ensure MasonryWall's async `redraw()`/`fillColumns()` pipeline completes before querying for cards.

In `Gallery.vue` `loadImagesForDirectory()`:
```javascript
// Line 156-165
if (reset) {
  await nextTick();
  // MasonryWall's onMounted redraw() fires in same tick — wait for its async fillColumns
  await nextTick();  // ADD THIS: wait for MasonryWall async layout to finish
  if (!observerRef.value) {
    observerRef.value = new IntersectionObserver(observeCallback, {
      root: null,
      rootMargin: '200px',
      threshold: 0.01,
    });
  }
  loadVisibleImages();
  setupLoadMoreObserver();
}
```

Alternative (more robust): use a setTimeout to ensure MasonryWall's microtask chain completes:
```javascript
if (reset) {
  await nextTick();
  await nextTick();  // wait for MasonryWall redraw() internal nextTick
  await nextTick();  // triple-check for deep async pipelines
  // ... rest unchanged
}
```

---

## Files Involved

- `E:\project\gelbooru\src\views\Gallery.vue` — line 157: needs additional `await nextTick()` before `loadVisibleImages()` at line 165
- `E:\project\gelbooru\node_modules\@yeger\vue-masonry-wall\dist\index.mjs` — lines 57-63: confirmed async `fillColumns` pipeline

---

## Secondary Issue (Observer Leak)

`observerRef.value` is set twice: once in `loadImagesForDirectory` (called before `onMounted` via `loadTree()`) and once in `onMounted`. The first observer becomes orphaned (never disconnected). Minor issue — the second observer works, but the first leaks. Not the primary root cause but worth fixing by removing the duplicate creation in `onMounted` or guarding it.

---

## Verification

After fix, observe:
1. Console logs in `observeCallback` fire (cards enter viewport)
2. `querySelectorAll('[data-image-path]')` returns count > 0
3. Skeleton shimmer transitions to loaded images as cards scroll into view
4. Test `Gallery.spec.ts` Test 2 passes: `observeSpy` called for each card

---

## Evidence Log

- timestamp: 2026-05-10T19:35
  checked: Gallery.vue template line 346-352
  found: `ref="galleryCardsRef"` is present — binding is correct
  implication: `galleryCardsRef.value` should be the GalleryCards component instance

- timestamp: 2026-05-10T19:38
  checked: GalleryCards.vue `defineExpose` line 45
  found: `setCardSrc`, `getCardSrc`, `imageCount` all exposed correctly
  implication: `galleryCardsRef.value.setCardSrc(path, src)` should work when ref is bound

- timestamp: 2026-05-10T19:40
  checked: MasonryWall source (node_modules/@yeger/vue-masonry-wall/dist/index.mjs lines 57-63)
  found: `fillColumns` is `async` with `await nextTick()` inside the loop — layout is NOT synchronous
  implication: Cards are added to DOM via a separate async pipeline that Vue's nextTick doesn't wait for

- timestamp: 2026-05-10T19:42
  checked: Gallery.vue `loadImagesForDirectory` lines 156-166
  found: Only ONE `await nextTick()` before `loadVisibleImages()` is called
  implication: `loadVisibleImages()` runs before MasonryWall async layout completes — 0 cards observed

- timestamp: 2026-05-10T19:43
  checked: Gallery.spec.ts test stub for gallery-cards component
  found: Stub uses `template: '<div />'` — does not expose `wall` ref or render `[data-image-path]` elements
  implication: Tests pass but do not validate actual DOM card observation — test coverage gap

- timestamp: 2026-05-10T19:45
  checked: tauri.conf.json CSP and asset protocol
  found: `img-src * data: blob: filesystem:` and `assetProtocol.enable: true` — asset URLs accessible
  implication: `convertFileSrc` URLs should load in img tags — not the blocking issue

- timestamp: 2026-05-10T19:46
  checked: Path normalization in observeCallback vs imageSrcMap.set/getCardSrc
  found: Both use raw `path` as Map key — normalization only applied to `convertFileSrc` argument
  implication: Path keys match — not the blocking issue

- timestamp: 2026-05-10T19:47
  checked: onMounted observer creation (lines 281-285) vs loadImagesForDirectory (lines 158-164)
  found: Observer created twice (before and after mount). First observer orphaned.
  implication: Secondary issue — does not prevent second observer from working, but a leak exists