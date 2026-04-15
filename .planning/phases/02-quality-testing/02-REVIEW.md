---
phase: 02-quality-testing
reviewed: 2026-04-15T00:00:00Z
depth: standard
files_reviewed: 3
files_reviewed_list:
  - src-tauri/Cargo.toml
  - src-tauri/src/db/mod.rs
  - src-tauri/src/services/scraper.rs
findings:
  critical: 0
  warning: 2
  info: 2
  total: 4
status: issues_found
---

# Phase 02: Code Review Report

**Reviewed:** 2026-04-15T00:00:00Z
**Depth:** standard
**Files Reviewed:** 3
**Status:** issues_found

## Summary

Three source files reviewed at standard depth. The code is generally well-structured with good test coverage and parameterized SQL queries that prevent injection. Two production-code concerns were found: debug print statements left in parsing logic, and a redundant local URL-encoding module that duplicates the already-declared `url` crate dependency. The `db/mod.rs` is clean. No security vulnerabilities, no hardcoded secrets, and no logic errors were identified.

## Warnings

### WR-01: Debug println statements left in production code

**File:** `src-tauri/src/services/scraper.rs`
**Lines:** 50-53, 67, 72, 109-112, 262-265, 279, 344-347, 361-362

Multiple `println!("[DEBUG] ...")` calls are present in the parsing functions `parse_pagination`, `parse_post_statistics`, and related helpers. These were likely added during development and left behind.

- Lines 50-53: inside `parse_pagination` loop
- Line 67: pagination container debug log
- Line 72: pagination link debug log
- Lines 109-112: "next page" debug log
- Lines 262-265: sample URL debug log
- Line 279: og:image debug log
- Lines 344-347: original image link debug log
- Lines 361-362: final URL debug logs

**Impact:** Clutters stdout, may interfere with structured logging pipelines, and is inappropriate for production code.

**Fix:** Remove all `println!("[DEBUG] ...")` calls. Replace with `tracing::debug!` macro (requires adding `tracing` to `[dependencies]`) or remove entirely:

```rust
// Before
println!("[DEBUG] Found page {} from pid {} in href: {}", page, pid, href);

// After
// (delete the line entirely, or use tracing if logging is needed)
```

### WR-02: Redundant local `urlencoding` module duplicates `url` crate dependency

**File:** `src-tauri/src/services/scraper.rs`
**Lines:** 418-422

The local `mod urlencoding` is a thin wrapper around `url::form_urlencoded::byte_serialize`:

```rust
mod urlencoding {
    pub fn encode(s: &str) -> String {
        url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
    }
}
```

The `url` crate is already listed in `[dependencies]` (Cargo.toml line 26) and is used directly inside this very module (line 420). The local module adds indirection without any additional logic.

**Fix:** Remove the entire `mod urlencoding { ... }` block (lines 418-422) and update the call site at line 402 to use the `url` crate directly:

```rust
// Before (line 402)
urlencoding::encode(&all_tags.join(" "))

// After
url::form_urlencoded::byte_serialize(all_tags.join(" ").as_bytes()).collect::<String>()
```

## Info

### IN-01: Unused `new` constructor

**File:** `src-tauri/src/services/scraper.rs`
**Lines:** 13-15

`GelbooruScraper::new()` exists but the struct has no fields, so it simply returns `Self`. The unit struct (line 10) is stateless. The constructor is only used in tests, not in production code paths.

**Suggestion:** The constructor is harmless but unnecessary. Consider removing it and using `Self` or `Default::default()` directly in tests. Alternatively, add `#[derive(Default)]` to the struct and remove the `impl Default` block at lines 412-415.

### IN-02: `PAGE_SIZE` constant is tightly coupled to Gelbooru's pagination

**File:** `src-tauri/src/services/scraper.rs`
**Lines:** 5, 47, 78, 106

`const PAGE_SIZE: u32 = 42;` is used in three places within `parse_pagination` to reverse-engineer page numbers from `pid` URL parameters (`pid / PAGE_SIZE + 1`). If Gelbooru ever changes their page size, this constant silently becomes incorrect and pagination will be wrong.

**Suggestion:** Document the assumption inline:

```rust
// PAGE_SIZE must match Gelbooru's posts-per-page (currently 42).
// If pagination page numbers become inaccurate, check this value first.
const PAGE_SIZE: u32 = 42;
```

---

_Reviewed: 2026-04-15T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
