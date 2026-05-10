---
name: gallery-thumbnails-not-loading
description: 图库卡片已显示（骨架 shimmer）但缩略图不加载
type: debug_session
status: resolved
created: 2026-05-10T09:17:00.000Z
updated: 2026-05-10T17:19:00.000Z
---

## Current Focus

**ROOT CAUSE CONFIRMED: `galleryCardsRef` is never bound to `<GalleryCards>` in the template. The ref is declared but never assigned — it is always `null` at runtime.**

## Hypothesis Tested

| # | Hypothesis | Test | Result |
|---|------------|------|--------|
| H1 | observerRef is null because refresh() clears it and loadImagesForDirectory doesn't recreate it | Read code lines 154-165 | ELIMINATED — guard `if (!observerRef.value)` handles null correctly |
| H2 | `galleryCardsRef` is not bound in template, so `setCardSrc` is never called | Read template lines 344-349 | **CONFIRMED — no `ref="galleryCardsRef"` attribute on `<GalleryCards>`** |
| H3 | `observeCallback` never fires at all | Code review of observer lifecycle | ELIMINATED — observer is created on mount and loadVisibleImages is called |
| H4 | `convertFileSrc` returns an inaccessible URL | CSP config review | ELIMINATED — CSP `img-src * data: blob: filesystem:` allows all, asset protocol enabled |
| H5 | `data-image-path` attribute not set on cards | Grep codebase | ELIMINATED — attribute is correctly set on line 71 of GalleryCards.vue |

---

## Root Cause

**`galleryCardsRef` is declared but never bound to the `<GalleryCards>` component.**

In `Gallery.vue`:
- Line 33: `const galleryCardsRef = ref<InstanceType<typeof GalleryCards> | null>(null);` — declares the ref
- Line 54: `galleryCardsRef.value?.setCardSrc(path, src);` — attempts to use it
- Lines 344-349: `<GalleryCards :images="images" ... />` — **MISSING `ref="galleryCardsRef"`**

Since the `ref` attribute is absent, `galleryCardsRef.value` is always `null`, so `galleryCardsRef.value?.setCardSrc(path, src)` is a **no-op** (optional chaining on null).

The `imageSrcMap` in `GalleryCards.vue` is never updated. The `v-if="getCardSrc(item.path)"` condition is always falsy. The shimmer placeholder is permanently visible. No `console.error` occurs because optional chaining silently swallows the null.

---

## Evidence

- **Debug file read:** `galleryCardsRef` used on line 54, declared on line 33, but no binding in template (lines 344-349)
- **Optional chaining:** `galleryCardsRef.value?.setCardSrc(...)` silently returns `undefined` when ref is null — no error thrown
- **Shimmer persists:** `getCardSrc(item.path)` returns `''` (empty string from Map.get with missing key), so `v-if="''"` is falsy — placeholder always shows
- **No console errors:** Optional chaining on null refs is valid JS; no exception is thrown
- **Timeline matches:** Recent gallery refactor likely moved or renamed the GalleryCards component template without restoring the `ref` binding

---

## Fix Direction

Add `ref="galleryCardsRef"` to the `<GalleryCards>` element in the template:

```vue
<GalleryCards
  ref="galleryCardsRef"
  :images="images"
  :loading-images="loadingImages"
  :selected-key="selectedKey"
  @open-preview="openPreview"
/>
```

After this fix, `galleryCardsRef.value` will be the `GalleryCards` component instance, and `setCardSrc` will correctly update `imageSrcMap`, causing `getCardSrc(item.path)` to return a non-empty URL, making the `<img>` render.

---

## Files Involved

- `E:\project\gelbooru\src\views\Gallery.vue` — line 344: missing `ref="galleryCardsRef"` on `<GalleryCards>`