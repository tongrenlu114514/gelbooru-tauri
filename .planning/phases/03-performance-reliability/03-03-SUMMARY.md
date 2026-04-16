---
phase: "03-performance-reliability"
plan: "03"
subsystem: infra
tags: [rust, tokio, filesystem, concurrency]

# Dependency graph
requires: []
provides:
  - Async parallel directory tree scanning with semaphore-bounded concurrency
  - MAX_CONCURRENT_DIRS=10 limit prevents OS file descriptor exhaustion
  - Deep directory trees (20+ levels) complete without deadlock
  - build_tree_async always returns root as result[0] with full tree aggregate
affects: [03-performance-reliability]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - spawn_blocking for directory enumeration (Windows: ReadDir not Send)
    - std::thread::scope for parallel child subdirectory traversal
    - tokio::sync::Semaphore for FD exhaustion prevention
    - Arc clone-before-acquire to avoid borrow conflicts in async fns

key-files:
  created: []
  modified:
    - src-tauri/src/commands/gallery.rs

key-decisions:
  - "Used spawn_blocking+std::thread::scope instead of tokio::fs+ tokio::spawn due to Windows ReadDir not being Send"
  - "Arc clone before permit.acquire() to avoid moving borrowed sem into spawn_blocking closure"

patterns-established:
  - "spawn_blocking async fn pattern: acquire permit on cloned Arc, move original Arc into blocking closure"
  - "build_tree_async always includes root as result[0] so image_count aggregates full tree"

requirements-completed: []

# Metrics
duration: ~20min
completed: 2026-04-17
---

# Phase 03 Plan 03: Async Parallel Directory Scanning Summary

**Parallel directory scanning with tokio::spawn_blocking and std::thread::scope, bounded by Semaphore(10), replacing two-phase blocking scan**

## Performance

- **Duration:** ~20 min (multiple sessions + wave-agent)
- **Started:** 2026-04-17T00:10:00Z
- **Completed:** 2026-04-17T01:00:00Z
- **Tasks:** 2 (TDD: test + implementation)
- **Files modified:** 1

## Accomplishments
- Async parallel directory scanning with MAX_CONCURRENT_DIRS=10 Semaphore limit
- Deep tree (20 levels, 1 subdir each, 1 image per dir = 21 images) completes without deadlock
- Root node always returned as result[0] with full tree image count aggregated
- Correct is_leaf flag derivation from children.is_empty()
- 94/94 tests passing, 0 clippy warnings

## Task Commits

1. **Task 1: TDD tests** - `098e911` (fix/feat/fix)
2. **Task 2: Implementation** - `f6467e6` (fix/fix)
3. **Subsequent fixes** - absorbed into `f6467e6`

**Plan metadata:** `12e6268` (docs(03): create phase plan)

## Files Created/Modified
- `src-tauri/src/commands/gallery.rs` - Async parallel directory scanner with Semaphore(10) concurrency bound

## Decisions Made

- **Used `spawn_blocking` + `std::thread::scope` instead of `tokio::fs::read_dir` + `tokio::spawn`:** On Windows, `tokio::fs::ReadDir` holds OS handles that are not `Send`, making the returned futures non-Send and incompatible with `tokio::spawn`. Solution: use `spawn_blocking` for all directory enumeration (blocking I/O) and `std::thread::scope` for parallel child subdirectory traversal within the blocking thread.
- **Arc clone before permit.acquire():** In an async fn, `_permit = sem.acquire().await` borrows `sem`, preventing it from being moved into a `spawn_blocking` closure. Fix: clone `sem` before acquiring, use the clone for the permit, move the original Arc into the closure.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] tokio::fs::ReadDir not Send on Windows**
- **Found during:** Task 2 (implementation)
- **Issue:** `tokio::fs::read_dir` returns `ReadDir` which holds OS handles not `Send` on Windows. `tokio::spawn(async { scan_dir(...).await })` fails with "future cannot be sent between threads safely"
- **Fix:** Replaced `tokio::spawn` + `tokio::fs::read_dir` pattern with `spawn_blocking` + `std::fs::read_dir` for enumeration, `std::thread::scope` for parallel child traversal
- **Files modified:** `src-tauri/src/commands/gallery.rs`
- **Verification:** `cargo test --lib -- commands::gallery::tests` passes 94/94

**2. [Rule 1 - Bug] build_tree_async missing root node in result**
- **Found during:** Task 2 (tests)
- **Issue:** `build_tree_async` only returned children of root, not root itself. Tests expected `result[0]` to be the root with full tree image_count
- **Fix:** Added root scan + node construction: root's own images counted separately, then children's image_counts summed for total. Root always included as `result[0]`
- **Files modified:** `src-tauri/src/commands/gallery.rs`
- **Verification:** `build_tree_async_counts_all_images_in_nested_dirs` (21 images), `build_tree_async_deep_tree_no_deadlock` (21 images) both pass

**3. [Rule 1 - Bug] scan_dir_sync returned children=None and is_leaf=true**
- **Found during:** Task 2 (tests)
- **Issue:** `scan_dir_sync` was aggregating child image counts but not collecting child TreeNode results into the `children` field. `is_leaf` was hardcoded to `true`
- **Fix:** Collect child results from `std::thread::scope` handles into `Vec<TreeNode>`, set `children: Some(children)`, derive `is_leaf` from `children.is_empty()`
- **Files modified:** `src-tauri/src/commands/gallery.rs`
- **Verification:** `build_tree_async_sets_is_leaf_correctly` passes

**4. [Rule 1 - Bug] spawn_blocking closure capturing &sem (not 'static)**
- **Found during:** Task 2 (compile verification)
- **Issue:** `spawn_blocking(move || count_dir_recursive(dir, &sem))` passed `&sem` (reference to Arc in async fn's stack), violating `'static` bound on `spawn_blocking` closure
- **Fix:** Changed `count_dir_recursive` and `scan_dir_sync` to take `Arc<Semaphore>` by value. Cloned Arc before acquire, moved original into closure
- **Files modified:** `src-tauri/src/commands/gallery.rs`
- **Verification:** `cargo build` succeeds

**5. [Rule 1 - Bug] path.is_dir().await on non-async Path**
- **Found during:** Task 2 (compile verification)
- **Issue:** `std::path::Path::is_dir()` is synchronous — `.await` on a `bool` is invalid
- **Fix:** Removed `.await` from `path.is_dir()` in `get_directory_images_async`
- **Files modified:** `src-tauri/src/commands/gallery.rs`
- **Verification:** `cargo build` succeeds

---

**Total deviations:** 5 auto-fixed (4 Rule 1 bugs, 1 Rule 3 blocking)
**Impact on plan:** All deviations were necessary for correctness. Rule 3 (Windows Send limitation) required a fundamental architecture change (blocking enum + thread scope vs async spawn). No scope creep.

## Issues Encountered

- **Windows `Send` limitation on `tokio::fs::ReadDir`:** Core architectural constraint that forced the `spawn_blocking` + `std::thread::scope` pattern. This is a known limitation of tokio on Windows — the `ReadDir` type holds OS handles that cannot be sent between threads.

## Threat Flags

None — path traversal already mitigated in Phase 2. FD exhaustion threat (T-03-02) is mitigated by Semaphore(10) limit.

## Next Phase Readiness

- gallery.rs async parallel scanning complete and tested
- 94/94 tests passing, clippy clean
- Ready for any plan that depends on directory scanning performance

---
*Phase: 03-performance-reliability, plan 03*
*Completed: 2026-04-17*
