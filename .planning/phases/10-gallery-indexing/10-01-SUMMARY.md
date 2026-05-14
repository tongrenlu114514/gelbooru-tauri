---
phase: "10"
plan: "01"
type: execute
wave: "1"
subsystem: "backend-indexing"
tags:
  - "gallery-indexing"
  - "thumbnail-generation"
  - "sqlite-index"
  - "tauri-commands"
dependency_graph:
  requires: []
  provides:
    - "002_gallery_index migration"
    - "5 Tauri commands (scan_gallery, get_indexed_images, generate_thumbnail, get_thumbnail_path, start_background_thumbnail_scan)"
    - "IndexingService background thumbnail queue"
  affects:
    - "src-tauri/src/commands/indexing.rs"
    - "src-tauri/src/db/mod.rs"
    - "src-tauri/Cargo.toml"
    - "src-tauri/src/commands/gallery.rs"
tech_stack:
  added:
    - "image = 0.25 (thumbnail generation)"
  patterns:
    - "Bounded mpsc channel for non-blocking thumbnail queue"
    - "spawn_blocking for CPU-bound image resize operations"
    - "Parameterised SQLite queries for gallery image indexing"
    - "DbState guard pattern (same as gallery.rs)"
key_files:
  created:
    - "src-tauri/src/commands/indexing.rs"
  modified:
    - "src-tauri/Cargo.toml"
    - "src-tauri/src/db/mod.rs"
    - "src-tauri/src/commands/mod.rs"
    - "src-tauri/src/commands/gallery.rs"
decisions:
  - id: "IDX-01"
    requirement: "SQLite index of local images"
    implementation: "gallery_images table via 002_gallery_index migration with upsert pattern"
  - id: "IDX-02"
    requirement: "Thumbnails in cache directory"
    implementation: "app_data_dir/thumbnails/ via IndexingService with path hashing"
  - id: "IDX-03"
    requirement: "On-demand thumbnail generation"
    implementation: "generate_thumbnail Tauri command with pre-queuing of other size"
  - id: "IDX-04"
    requirement: "Background thumbnail generation"
    implementation: "IndexingService with bounded mpsc channel (size=100) + spawn_blocking"
metrics:
  duration: ""
  completed: "2026-05-15"
---

# Phase 10 Plan 1: Gallery Indexing Backend - Summary

## One-liner

Backend gallery indexing infrastructure: SQLite gallery_images + thumbnails tables, 5 Tauri commands for scanning/pagination/thumbnail generation, and IndexingService background queue using bounded mpsc channel.

## Commits

| # | Commit | Description |
|---|--------|-------------|
| 1 | `ae150c2` | feat(10-01): add image crate 0.25 for thumbnail generation |
| 2 | `9b94735` | feat(10-01): add 002_gallery_index migration with gallery_images + thumbnails tables and helper methods |
| 3 | `58e858b` | feat(10-01): create indexing.rs with all 5 Tauri commands and IndexingService |
| 4 | `3075130` | fix(10-01): fix compilation errors in indexing module |

## Completed Tasks

### Task 1: Add image crate dependency to Cargo.toml
- **Commit:** `ae150c2`
- **Files:** `src-tauri/Cargo.toml`
- **Action:** Added `image = "0.25"` to dependencies
- **Verification:** `cargo check` passes, 105 tests pass

### Task 2: Add 002_gallery_index migration to db/mod.rs
- **Commit:** `9b94735`
- **Files:** `src-tauri/src/db/mod.rs`
- **Action:** Added migration with `gallery_images` + `thumbnails` tables, plus 6 DB helper methods: `upsert_gallery_image`, `get_gallery_image_by_path`, `get_indexed_images`, `save_thumbnail_entry`, `get_thumbnail_entry`, `update_gallery_thumbnail_path`, `get_images_without_thumbnails`
- **Verification:** `cargo check` passes

### Task 3: Create src-tauri/src/commands/indexing.rs with commands + thumbnail service
- **Commit:** `58e858b` (initial), `3075130` (fix)
- **Files:** `src-tauri/src/commands/indexing.rs`, `src-tauri/src/commands/mod.rs`, `src-tauri/src/commands/gallery.rs`
- **Action:** Created indexing.rs with 5 Tauri commands + IndexingService, added `pub mod indexing` to mod.rs, made `is_image` pub(crate) in gallery.rs
- **Verification:** `cargo check` passes (0 errors, 23 warnings for unused items — expected since Wave 2 frontend not yet wired)

### Task 4: Verify backend compiles and tests pass
- **Verification:** `cargo check` exits 0, `cargo test` all 105 tests pass

## What Was Built

### Database Schema (002_gallery_index)
- `gallery_images` table: id, file_path (unique), file_name, file_size, width, height, file_hash, thumbnail_path, last_scanned, created_at
- `thumbnails` table: id, image_id (FK), width, height, thumbnail_path, created_at
- Indexes on file_path and image_id for query performance

### Tauri Commands
| Command | Purpose |
|---------|---------|
| `scan_gallery` | Recursively index directory, upsert to gallery_images, queue background thumbnails |
| `get_indexed_images` | Paginated query from gallery_images table |
| `generate_thumbnail` | On-demand thumbnail with pre-queuing of other size |
| `get_thumbnail_path` | Size-specific thumbnail retrieval wrapper |
| `start_background_thumbnail_scan` | Bulk queue all images missing thumbnails |

### IndexingService
- Bounded mpsc channel (size=100) for non-blocking queue
- `spawn_blocking` for CPU-bound Lanczos3 resize
- Auto-saves thumbnail entry + updates gallery_images.thumbnail_path on success

## Deviations from Plan

### Auto-fixed Issues (Rule 1 - Bug)

**1. Private `is_image` function in gallery.rs**
- **Found during:** Task 3
- **Issue:** `is_image` was `fn` (private) — indexing.rs couldn't re-export it
- **Fix:** Changed to `pub(crate) fn is_image` in gallery.rs
- **Files modified:** `src-tauri/src/commands/gallery.rs`
- **Commit:** `3075130`

**2. Missing `tauri::Manager` trait import**
- **Found during:** Task 3 (compilation)
- **Issue:** `try_state`, `path()`, `manage` methods require `Manager` trait in scope
- **Fix:** Added `use tauri::Manager;` to indexing.rs
- **Commit:** `3075130`

**3. Missing `image::GenericImageView` trait import**
- **Found during:** Task 3 (compilation)
- **Issue:** `img.dimensions()` requires `GenericImageView` trait
- **Fix:** Added `GenericImageView` to image crate import
- **Commit:** `3075130`

**4. Borrow-after-move in IndexingService**
- **Found during:** Task 3 (compilation)
- **Issue:** Paths cloned before `spawn_blocking` but still used after await — compiler error
- **Fix:** Triple-clone pattern: clone for closure + clone for post-await use
- **Commit:** `3075130`

### Rule 2 - Auto-add Missing Critical Functionality

**5. Added `get_images_without_thumbnails` DB helper**
- **Issue:** `start_background_thumbnail_scan` needed direct access to `db.conn` private field
- **Fix:** Added helper method to Database struct, eliminating the private field access
- **Files modified:** `src-tauri/src/db/mod.rs`
- **Commit:** `3075130`

### Rule 3 - Auto-fix Blocking Issues

**6. Path type mismatch in start_background_thumbnail_scan**
- **Found during:** Task 3 (compilation)
- **Issue:** `query_map` returned `(_, str)` instead of `(_, String)` — `conn` field private
- **Fix:** Used new `get_images_without_thumbnails` helper method that returns `Vec<(i64, String)>`

## Threat Surface

| Flag | File | Description |
|------|------|-------------|
| none | - | No new security surface introduced — all paths validated via `walkdir.follow_links(false)`, DB uses parameterised queries, thumbnail files written only to `app_data_dir/thumbnails/` |

## Known Stubs

None — all DB helper methods and Tauri commands are fully implemented. Unused warnings in `cargo check` are expected since Wave 2 (frontend) will wire up the commands.

## Self-Check: PASSED

- `cargo check` exits 0 (0 errors, 23 warnings for unused items)
- `cargo test` all 105 tests pass
- 4 commits created with correct Conventional Commits format
- All files from plan created/modified