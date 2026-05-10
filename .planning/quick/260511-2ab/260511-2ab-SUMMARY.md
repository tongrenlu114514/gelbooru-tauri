# Quick Task 260511-2ab: 删除图片后不要调用refresh，直接splice移除列表项

**Date:** 2026-05-10
**Commit:** ac47cde
**Status:** COMPLETED

## Summary

- `deleteImage`: 删除成功后直接 `splice` 移除列表项，不调用 `refresh()`
- `refresh()` 只在面包屑导航切换目录时才调用
- 改为 Promise chain（`.then()/.catch()`）处理异步，避免 async/await 闭包问题

## Verification

- 删除图片后列表瞬时更新，无闪烁/白屏
- 预览模式同步关闭