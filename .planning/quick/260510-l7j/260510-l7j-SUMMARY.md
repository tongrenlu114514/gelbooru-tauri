# Quick Task 260510-l7j: 瀑布流直接显示图库里的图片，不要用文件夹聚合

**Executed:** 2026-05-10
**Status:** completed

## Summary

移除了文件夹分组逻辑，瀑布流直接显示图库目录下所有图片（递归收集），按修改时间倒序排列。面包屑导航保持不变。

## Changes

### Backend (Rust)

**`src-tauri/src/commands/gallery.rs`**
- 重写 `get_directory_images_async`：递归收集目录下所有图片（含子目录），按修改时间倒序排列，`subdirs` 恒返回空数组
- 新增 `collect_images_recursive` 辅助函数：使用 `walkdir` 递归遍历，收集图片路径和修改时间
- 更新测试：`get_directory_images_async_returns_flat_images_sorted_by_mtime` 验证递归收集 + 无文件夹分组

**`src-tauri/Cargo.toml`**
- 新增依赖：`walkdir = "2"`（递归目录遍历）
- 新增 dev-dependency：`filetime = "0.2"`

### Frontend (Vue)

**`src/views/GalleryCards.vue`**
- `displayItems` 不再混合 subdirs，直接返回纯图片列表
- 模板中移除 `v-if="item._type === 'folder'"` 分支
- 移除 `FolderOutline` 图标导入
- 移除 `.folder-card`、`.folder-preview` 等文件夹相关 CSS

### Build & Tests

- `cargo build` ✓
- `cargo test get_directory_images_async` ✓ (1 passed)
- `pnpm build` ✓
- `pnpm vitest run` ✓ (118 passed)

## Key Decisions

- 使用 `walkdir` 而非递归 async fn（避免 `E0733: recursion in an async fn requires boxing`）
- 保留 `subdirs` 字段在 API 响应中（空数组），维持响应类型兼容性
- 面包屑逻辑不变：`enterSubdir` 仍能通过 breadcrumb 点击触发导航

## Commit

`118801b` — `feat(gallery): flat masonry with recursive images sorted by mtime`