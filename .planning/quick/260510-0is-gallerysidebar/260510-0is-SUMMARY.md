---
quick_id: "260510-0is"
status: complete
started: "2026-05-10T00:38:00Z"
completed: "2026-05-10T00:42:00Z"
wave: 1
tasks_total: 2
tasks_completed: 2
---

## Quick Task Summary

**删除侧边栏，改为横向分割线区分的扁平文件夹列表**

## What Was Built

Removed the collapsible 240px sidebar (GallerySidebar.vue) and replaced it with a flat inline folder list rendered inside the content area above the image grid.

### Changes

**Gallery.vue (460 lines — was 371):**
- Removed `n-layout has-sider` → plain `n-layout` (full width content)
- Removed `import GallerySidebar` and `useSettingsStore` (sidebar no longer used)
- Removed: `treeData`, `loadingTree`, `TreeNode` interface, `findNodeByKey`, `handleTreeSelect`
- Added flat folder list section above GalleryCards:
  - ".." up-navigation entry (ChevronUpOutline icon)
  - Subdir entries: folder icon + name + image count badge
  - `|` horizontal dividers between items
  - Click ".." → navigate to parent path
  - Click subdir → `enterSubdir(subdir)` → load its images and subdirs
- Added `parentPath` computed: derives parent by stripping last path segment
- Added `goUp()` function for ".." navigation
- Loading state via NSpin around folder list
- Empty state: "从上方列表选择一个文件夹开始浏览"
- Preserved: preview modal, keyboard nav (ArrowLeft/Right/Escape), IntersectionObserver, delete confirmation

**GallerySidebar.vue:** DELETED

**src/tests/setup.ts:** Updated Pinia setup comment (removed "and GallerySidebar.vue")

**src/views/Gallery.spec.ts:** Removed `gallery-sidebar` stub (no longer referenced)

## Verification

- `pnpm vitest run src/views/Gallery.spec.ts`: **8/8 passed**
- `pnpm vitest run`: **118/118 passed**
- `grep "GallerySidebar" src/`: only comment reference in setup.ts (updated)
- `grep "gallery-sidebar" src/`: 0 references in source
- Build: linting clean

## Must-Haves

| Truth | Status |
|-------|--------|
| Folder list replaces sidebar — no 240px collapsible panel | ✓ |
| Current directory's subfolders shown as horizontal flat list above image grid | ✓ |
| Clicking a folder navigates into it and updates the flat list | ✓ |
| Up navigation available via ".." entry in folder list | ✓ |