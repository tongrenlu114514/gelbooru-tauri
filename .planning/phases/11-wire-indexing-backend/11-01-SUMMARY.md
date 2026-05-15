---
phase: "11"
plan: "01"
status: complete
subsystem: tauri-backend
tags: [indexing, wiring, backend]
key-files:
  created: []
  modified:
    - src-tauri/src/main.rs
    - src-tauri/src/commands/indexing.rs
    - src-tauri/src/commands/favorite_tags.rs
metrics:
  commands_wired: 5
  files_changed: 3
  commits: 1
---

## Plan 11-01: Wire Indexing Backend — Complete

### What Was Built

Phase 10's IndexingService and all 5 indexing Tauri commands are now registered and functional at runtime.

### Commits

| Task | Commit | Description |
|------|--------|-------------|
| Task 1 | `e5d4bc2` | Wire IndexingService + 5 commands in main.rs |

### Changes

**src-tauri/src/main.rs**
- `Arc::new(Database)` wraps DB from the start — shared between `DbState` and `IndexingService`
- `.setup()` closure calls `setup_indexing_service()` with `database.clone()`
- Graceful degrade via `catch_unwind` + `AssertUnwindSafe` — errors logged, app continues
- All 5 Phase 10 commands registered in `generate_handler` under `// indexing (Phase 11)` block

**src-tauri/src/commands/favorite_tags.rs**
- `DbState` changed from `Mutex<Database>` to `Mutex<Arc<Database>>`

**src-tauri/src/commands/indexing.rs**
- Fixed Send bound in `generate_thumbnail`: extracted all DB data before `await` into a block-scoped scope, preventing `MutexGuard` from crossing the await boundary

### Decisions Applied

| ID | Decision | Implementation |
|----|----------|----------------|
| D-01 | Graceful degrade | `catch_unwind` + `AssertUnwindSafe`, error logged to stderr |
| D-02 | Module-commented style | `// indexing (Phase 11)` comment block |
| D-03 | Arc-from-start | `Arc::new(Database)` — DbState and IndexingService both use `Arc<Database>` |

### Deviations

- **Arc-from-start**: Original plan proposed passing plain `Database` to `DbState` and `Arc::new(database)` only for `IndexingService`. But `DbState` holds `Mutex<Database>`, so the Arc must wrap the entire database from the start — both consumers get `Arc::clone(&database)`.
- **catch_unwind fix**: Original used `.map_err()` directly on `catch_unwind` which doesn't work — switched to `if let Err(e) = ...` pattern with `AssertUnwindSafe`.

### Self-Check

- [x] `setup_indexing_service` in main.rs
- [x] All 5 `commands::indexing::` entries in generate_handler
- [x] `// indexing (Phase 11)` comment present
- [x] `catch_unwind` + `AssertUnwindSafe` present
- [x] `Arc::new` present
- [x] `cargo check` exits 0 (6 pre-existing warnings only)
- [x] Human verification: approved