# Phase 11: Wire Indexing Backend - Context

**Gathered:** 2026-05-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Register `IndexingService` and all 5 Phase 10 Tauri commands in `main.rs` so the Phase 10 backend is callable at runtime. This is a pure Rust wiring phase — no new features, no UI, gap closure only.

**Scope:**
1. Call `setup_indexing_service()` after `.manage(DbState(...))` in `main.rs`
2. Add 5 indexing commands to `generate_handler![...]` in `main.rs`
3. Verify `app_handle.try_state::<IndexingService>()` returns `Some` at runtime

</domain>

<decisions>
## Implementation Decisions

### Setup Failure Handling (D-01)
- **Graceful Degrade** — If `setup_indexing_service()` fails, the app continues running.
- IndexingService is unavailable at runtime (`try_state` returns `None`), but all other features (download, search, gallery browse) work normally.
- Thumbnail generation falls back to synchronous on-demand path (no background queue).
- Error is logged to stderr, not surfaced as a blocking dialog.

### Command Registration Style (D-02)
- **Module-commented style** — `generate_handler![...]` entries are grouped by module with inline `//` comments.
- New indexing block added as:
  ```rust
  // indexing (Phase 11)
  commands::indexing::scan_gallery,
  commands::indexing::get_indexed_images,
  commands::indexing::generate_thumbnail,
  commands::indexing::get_thumbnail_path,
  commands::indexing::start_background_thumbnail_scan,
  ```
- Matches existing codebase style where gelbooru, download, gallery, favorite_tags, settings each have their own block.

### IndexingService Registration (D-03)
- `setup_indexing_service()` called **after** `.manage(DbState(...))` and **before** `.invoke_handler(...)`.
- `Arc::new(database)` passed in — shares the same `Database` instance with `DbState`.
- No `expect()`/`.unwrap()` on setup — errors caught and logged.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 10 (prerequisite — files created in Phase 10)
- `src-tauri/src/commands/indexing.rs` — IndexingService struct, 5 Tauri commands, `setup_indexing_service()` function
- `src-tauri/src/db/mod.rs` — `002_gallery_index` migration, `gallery_images`/`thumbnails` tables, helper methods
- `src-tauri/src/commands/mod.rs` — `pub mod indexing;` (already added in Phase 10)
- `src-tauri/src/main.rs` — current state (14 commands, no indexing, no IndexingService setup)
- `.planning/phases/10-gallery-indexing/10-01-PLAN.md` — full Phase 10 spec including command signatures

### Milestone Audit (gap documentation)
- `.planning/v1.2-MILESTONE-AUDIT.md` — gap evidence: `setup_indexing_service` never called, 5 commands unregistered

</canonical_refs>

<codebase_context>
## Existing Code Insights

### Reusable Assets
- `commands/indexing.rs` — already exists with all 5 commands and `IndexingService`
- `db/mod.rs` — `Database` type, `Arc<Database>` available for sharing

### Established Patterns
- `main.rs` uses module-comment style for `generate_handler!` blocks (gelbooru, download, gallery, favorite_tags, settings)
- `DbState(Mutex::new(database))` pattern — `database` is a `Database` instance, same instance passed to IndexingService via `Arc::new`
- Graceful error handling in existing commands — `.map_err(|e| e.to_string())` pattern

### Integration Points
- `main.rs` line ~22: `manage(DbState(Mutex::new(database)))` — `setup_indexing_service` called immediately after here
- `main.rs` line ~26: `generate_handler![...]` — 5 indexing commands appended here

</codebase_context>

<specifics>
## Specific Ideas

- `setup_indexing_service(&app_handle, Arc::new(database))` — exact call signature
- Graceful degrade means `try_state::<IndexingService>()` returns `None` → existing `if let Some(svc) = app_handle.try_state()...` guards in commands handle it naturally
- No new frontend components, no new tests required at this phase (backend wiring only)

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

---

*Phase: 11-wire-indexing-backend*
*Context gathered: 2026-05-16*