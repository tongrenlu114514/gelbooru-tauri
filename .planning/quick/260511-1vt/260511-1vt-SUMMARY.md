# Quick Task 260511-1vt: 取消图片列表懒加载，图片改为懒加载

**Date:** 2026-05-10
**Commit:** b4ad728
**Status:** COMPLETED

## Summary

- `Gallery.vue`: `loadImagesForDirectory` 改为一次性加载全部图片（limit=10000），移除分页、IntersectionObserver、sentinel、load-more-ref
- `GalleryCards.vue`: `<img>` 添加 `loading="lazy"` 利用浏览器原生懒加载
- 删除了 169 行无用代码（Observer 逻辑、sentinel 模板/样式、page/hasMore 状态）

## Changes

| File | Change |
|------|--------|
| `src/views/Gallery.vue` | -169 lines: remove infinite scroll, pagination, IntersectionObserver |
| `src/views/GalleryCards.vue` | +1 line: `loading="lazy"` |
| `src/views/Gallery.spec.ts` | Remove unused `nextTick` import |

## Verification

- `pnpm tsc --noEmit` passes
- 浏览器打开 Gallery 页面，所有图片正常显示
- 滚动时浏览器自动懒加载视口外的图片