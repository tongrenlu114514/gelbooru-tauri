---
phase: "6"
plan: "01"
subsystem: gallery
tags: [masonry, gallery, layout, waterfall]
dependency_graph:
  requires: []
  provides: [masonry-grid, masonry-cards, masonry-skeleton]
  affects: [Gallery.vue]
tech_stack:
  added: ["@yeger/vue-masonry-wall@6.1.1"]
  patterns: [masonry-layout, discriminated-union-items]
key_files:
  created: []
  modified:
    - src/views/GalleryCards.vue
    - package.json
decisions:
  - id: "D-06-Masonry"
    description: "Use @yeger/vue-masonry-wall for true variable-height masonry layout instead of CSS Grid auto-fill"
  - id: "D-06-DisplayItems"
    description: "Combine subdirs and images into DisplayItem discriminated union with _type field for type-safe slot rendering"
---

# Phase 06 Plan 01: Masonry Grid for GalleryCards Summary

**Completed:** 2026-05-10
**Duration:** ~10 minutes

## One-liner

Replaced CSS Grid in GalleryCards.vue with @yeger/vue-masonry-wall for true variable-height waterfall layout, combining folders and images into a typed discriminated union.

## Tasks Completed

| Task | Commit | Files |
|------|--------|-------|
| Task 1: Install @yeger/vue-masonry-wall | 4597f6b | package.json |
| Task 2: Refactor GalleryCards.vue CSS Grid to MasonryWall | bfa8f1d | src/views/GalleryCards.vue |
| Restore @vue/test-utils (blocking issue) | c208640 | package.json |

## Changes

### Task 1: Install @yeger/vue-masonry-wall (4597f6b)

- Added `@yeger/vue-masonry-wall@6.1.1` to `dependencies` in `package.json`
- Transitive dependency `@yeger/debounce` installed automatically

### Task 2: Refactor GalleryCards.vue (bfa8f1d)

**Import:** Added `MasonryWall` import from `@yeger/vue-masonry-wall`

**Type:** Added `DisplayItem` discriminated union type combining `ImageInfo` and `SubDirInfo` with `_type: 'image' | 'folder'` discriminator.

**Computed:** Added `displayItems` computed property that:
- Maps `props.subdirs` to folder items (with `_type: 'folder'`)
- Maps `props.images` to image items (with `_type: 'image'`)
- Concatenates with folders first, images after

**Template:** Replaced CSS Grid `.content-grid` div with `<MasonryWall>` component:
- `:items="displayItems"`, `:column-width="160"`, `:gap="4"`, `:min-columns="1"`
- Slot renders folder cards (`v-if="item._type === 'folder'"`) and image cards (`v-else`)
- Folder cards: emit `enter-subdir` with folder object
- Image cards: emit `open-preview` with `index - props.subdirs.length` for correct image offset
- All card styles preserved (`hover` shadow, gradient, filename reveal)

**ResizeObserver removed:** MasonryWall handles responsive reflow internally; `skeletonCount` ref retained for loading skeleton.

**All styles preserved:** `.content-grid`, `.gallery-card`, `.card-filename`, `.folder-preview`, `.skeleton-card`, `.empty-state` unchanged.

## Deviations

**1. [Rule 3 - Blocking] Restored @vue/test-utils dev dependency**
- **Found during:** Task 2 verification (test run)
- **Issue:** `pnpm add @yeger/vue-masonry-wall` accidentally removed `@vue/test-utils` (dependency deduplication hoisting)
- **Fix:** Ran `pnpm add -D @vue/test-utils` to restore it
- **Files modified:** package.json
- **Commit:** c208640

## Test Results

- `pnpm vitest run src/views/Gallery.spec.ts`: 8/8 passed
- `pnpm vitest run` (full suite): 118/118 passed

## Acceptance Criteria

- [x] package.json contains "@yeger/vue-masonry-wall"
- [x] GalleryCards.vue imports MasonryWall from '@yeger/vue-masonry-wall'
- [x] GalleryCards.vue has DisplayItem type combining ImageInfo & SubDirInfo with _type field
- [x] GalleryCards.vue has displayItems computed merging subdirs + images with _type tags
- [x] GalleryCards.vue uses MasonryWall component with :column-width="160" :gap="4" :min-columns="1"
- [x] MasonryWall slot renders folder cards (v-if="item._type === 'folder'") and image cards (v-else)
- [x] @click handlers use index - props.subdirs.length for image preview index
- [x] All existing CSS styles (.gallery-card, .card-filename, .folder-preview, .skeleton-card, .empty-state) preserved
- [x] ResizeObserver setup removed from onMounted/onUnmounted
- [x] skeletonCount ref retained for loading skeleton display
- [x] `pnpm vitest run src/views/Gallery.spec.ts` passes

## Commits

- `4597f6b` feat(06-01): install @yeger/vue-masonry-wall
- `bfa8f1d` feat(06-01): refactor GalleryCards.vue CSS Grid to MasonryWall
- `c208640` chore: restore @vue/test-utils dev dependency
