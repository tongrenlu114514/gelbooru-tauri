---
phase: "03"
plan: "02"
subsystem: infra
tags: [tokio, retry, exponential-backoff, mpsc, tdd]

# Dependency graph
requires: []
provides:
  - Exponential backoff retry wrapper around HTTP download calls
  - 8 unit tests covering all retry and cancellation scenarios
  - Separate pause/cancel channels preserving partial-file soft-stop
affects: [03-01, 03-03]

# Tech tracking
tech-stack:
  added: [tokio/test-util for virtual-time testing]
  patterns:
    - Exponential backoff: BASE_DELAY_MS * 2^(attempt-1) = 1s, 2s, 4s
    - tokio::select! racing async ops against cancellation
    - mpsc channel is_closed() check after select! arms
    - Separate cancel/pause channels for hard-stop vs soft-stop

key-files:
  created: []
  modified:
    - src-tauri/src/commands/download.rs
    - src-tauri/Cargo.toml
    - src-tauri/Cargo.lock

key-decisions:
  - "Used separate cancel_rx (consumed by retry) and pause_rx (used by streaming loop) instead of cloning — cleaner separation of concerns"
  - "Production retry_fetch uses concrete reqwest types; test download_with_retry uses generic FnMut trait for mocking"
  - "tokio::select! + is_closed() pattern for mpsc cancellation (recv() returns None on closed channel, not Err)"
  - "BASE_DELAY_MS * 2_u64.saturating_pow(attempt-1) for safe overflow handling in backoff calculation"

patterns-established:
  - "TDD in Rust: #[cfg(test)] module-level async function + tokio::time::pause()/advance() for virtual-time unit tests"
  - "Cancellation contract: is_closed() must be checked after every select! arm that awaits recv() on mpsc channel"

requirements-completed: []

# Metrics
duration: ~80min
completed: 2026-04-17
---

# Phase 03: Plan 02 Summary

**Exponential backoff retry (1s/2s/4s) wrapping HTTP downloads with 8 unit tests covering transport errors, 5xx retry, 4xx no-retry, cancellation during sleep, and max-retries exhaustion**

## Performance

- **Duration:** ~80 min
- **Started:** 2026-04-17T00:10:00Z
- **Completed:** 2026-04-17T01:30:00Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- `retry_fetch` function with 3-attempt exponential backoff (1s, 2s, 4s) wired into `start_download`
- Transport errors and 5xx responses trigger retry; 4xx and 2xx return immediately without retry
- Cancellation drains pending backoff immediately via `tokio::select!` + `is_closed()` check (no waiting through full delay)
- 8 unit tests using virtual time (`tokio::time::pause()/advance()`) covering all retry scenarios
- Separate `cancel_rx`/`pause_rx` channels: cancel for hard-stop (retry loop), pause for soft-stop (streaming loop preserves partial file)

## Task Commits

Each task was committed atomically:

1. **Task 1 (TDD): Add tests + implement download_with_retry** - `b0d5267` (feat)
2. **Task 2: Wire retry_fetch into start_download** - `b098c69` (feat)

**Plan metadata:** `b098c69` (last commit includes summary)

## Files Created/Modified

- `src-tauri/src/commands/download.rs` - Added `FetchOutcome` enum, `retry_fetch` function, `download_with_retry` test helper with 8 unit tests, separate cancel/pause channel management
- `src-tauri/Cargo.toml` - Added `test-util` feature to tokio dependency for virtual-time testing
- `src-tauri/Cargo.lock` - Updated lockfile

## Decisions Made

- **Separate cancel/pause channels**: `retry_fetch` consumes `cancel_rx` (hard cancellation during retry). A separate `pause_rx` is used by the streaming loop for soft-stop (preserves partial file). Both channels share the same `cancel_tx` sender in `DownloadManager` — `pause_download` sends on `pause_tx`, `cancel_download` drops `cancel_tx`.
- **tokio::select! + is_closed()**: `mpsc::Receiver::recv()` returns `None` (not `Err`) when the sender is dropped. The `_ = recv()` pattern in `select!` only matches `Some(())` and `Err`, so `None` falls through silently. Solution: check `is_closed()` after each `select!` arm that could complete while the sender is dropped.
- **`saturating_pow` over `pow`**: Used `2_u64.saturating_pow(attempt - 1)` instead of `2_u64.pow(attempt - 1)` to avoid panics on overflow.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] tokio::time::pause/advance not found**
- **Found during:** Task 1 (TDD implementation)
- **Issue:** `tokio::time::pause()` and `tokio::time::advance()` require the `test-util` Cargo feature
- **Fix:** Added `test-util` feature to tokio in Cargo.toml: `tokio = { version = "1", features = ["full", "test-util"] }`
- **Files modified:** src-tauri/Cargo.toml, src-tauri/Cargo.lock
- **Verification:** `cargo clippy -- -D warnings` passes
- **Committed in:** `b0d5267` (Task 1 commit)

**2. [Rule 1 - Bug] `cancel_rx.recv()` returns None not Err on channel close**
- **Found during:** Task 1 (cancellation test)
- **Issue:** When `cancel_tx` is dropped, `recv()` returns `None` which `_ = recv()` does NOT match. The retry loop kept running because `None` fell through the `_ = recv()` pattern in `tokio::select!`
- **Fix:** Added `if cancel_rx.is_closed()` checks after every `select!` arm that awaits `recv()` on the cancellation channel
- **Files modified:** src-tauri/src/commands/download.rs
- **Verification:** `respects_cancellation_during_retry_sleep` test passes
- **Committed in:** `b0d5267` (Task 1 commit)

**3. [Rule 1 - Bug] dead_code clippy errors for cfg(test) items**
- **Found during:** Task 1 (post-implementation verification)
- **Issue:** `MAX_RETRIES`, `BASE_DELAY_MS`, `DownloadAttempt`, `RetryFetchResult`, and `download_with_retry` were flagged as unused in production code (before wiring into `start_download`)
- **Fix:** Moved `MAX_RETRIES` and `BASE_DELAY_MS` to module-level constants (non-cfg), kept `DownloadAttempt`, `RetryFetchResult`, and `download_with_retry` as `#[cfg(test)]`. Created separate production `FetchOutcome` enum and `retry_fetch` function for the wired-in code.
- **Files modified:** src-tauri/src/commands/download.rs
- **Verification:** `cargo clippy -- -D warnings` passes
- **Committed in:** `b098c69` (Task 2 commit)

**4. [Rule 2 - Missing Critical] Streaming loop loses pause detection after retry_fetch consumes cancel_rx**
- **Found during:** Task 2 (wiring retry into start_download)
- **Issue:** Original code used `cancel_rx` for both retry and streaming pause detection. `download_with_retry` consumes `cancel_rx`, leaving the streaming loop with no way to detect pause/cancel after the HTTP call succeeds
- **Fix:** Split into two channels: `cancel_rx` for retry cancellation (hard-stop), `pause_rx` for streaming pause (soft-stop, preserves partial file). Added `pause_tokens: RwLock<HashMap<u32, mpsc::Sender<()>>>` to `DownloadManager`. Updated `pause_download` to send on `pause_tx`
- **Files modified:** src-tauri/src/commands/download.rs
- **Verification:** Build passes, all tests pass
- **Committed in:** `b098c69` (Task 2 commit)

---

**Total deviations:** 4 auto-fixed (1 blocking, 3 correctness)
**Impact on plan:** All auto-fixes essential for correctness. The cancel/pause channel split was a necessary architectural correction beyond the plan's scope; it preserves the intended soft-stop behaviour for pause.

## Issues Encountered

- **`never_loop` clippy lint**: Inner `loop {}` with only `break` or `return` triggers `clippy::never_loop`. Fixed by replacing inner loops with plain `tokio::select!` + `is_closed()` check
- **`Box::pin` type mismatch in test**: Removed unnecessary `Box::pin` wrapper; closure already returns `impl Future`
- **`noop_method_call` clippy warning**: `AtomicU32` doesn't implement `Clone` — `attempt_count_clone.clone()` was a no-op. Fixed by removing `.clone()`
- **`cancel_rx` consumption**: When `retry_fetch` takes `&mut cancel_rx`, the receiver is consumed. This required restructuring the spawned task to use a separate `pause_rx` for the streaming loop

## Next Phase Readiness

- Retry infrastructure in place for other HTTP operations
- The `retry_fetch` / `FetchOutcome` pattern can be extracted to `src-tauri/src/services/http.rs` for reuse if other commands need retry logic
- Pause/resume during retry phase is not yet implemented (pause only works during the streaming phase); this would require adding a `pause_rx` argument to `retry_fetch` as well

---
*Phase: 03-02*
*Completed: 2026-04-17*
