# Codebase Concerns

**Analysis Date:** 2026-04-13

---

## Security Concerns

### [HIGH] Hardcoded Default Paths

**Issue:** Default paths are hardcoded in multiple locations, creating security and portability risks.

**Files affected:**
- `src-tauri/src/commands/gallery.rs` (lines 67, 141): `"D:/project/gelbooru/imgs/"`
- `src/stores/settings.ts` (line 7): `'D:/project/gelbooru/imgs/'`
- `src-tauri/src/services/http.rs` (line 15): `"http://127.0.0.1:7897"` (proxy URL)

**Impact:**
- Path traversal vulnerabilities possible if user input is not sanitized
- Application only works on this specific developer's machine
- Proxy configuration is exposed in source code

**Fix approach:** Move all defaults to configuration files (e.g., `tauri.conf.json` or `.env`). Add path validation before file operations.

---

### [MEDIUM] No Path Sanitization for File Operations

**Issue:** File paths are used directly without validation in gallery commands.

**Files affected:**
- `src-tauri/src/commands/gallery.rs`: `get_local_images`, `get_directory_tree`, `get_directory_images`, `get_local_image_base64`, `delete_image`
- `src-tauri/src/commands/download.rs`: `open_file`

**Impact:**
- Potential path traversal attacks (e.g., `../../etc/passwd`)
- No validation that paths are within allowed directories
- `delete_image` can delete any file the application has access to

**Fix approach:** Validate paths using canonicalization (`std::fs::canonicalize`) and verify they are within allowed directories.

---

### [MEDIUM] No Authentication or Rate Limiting

**Issue:** No authentication mechanism for Gelbooru API access, and no rate limiting implemented.

**Files affected:**
- `src-tauri/src/services/http.rs`: HTTP client with no rate limiting
- `src-tauri/src/commands/gelbooru.rs`: All search/download commands

**Impact:**
- API may block requests due to rate limiting
- No protection against abuse
- No cookie/session management for authenticated features

**Fix approach:** Implement request throttling and consider Gelbooru authentication if required.

---

### [LOW] Cookie Loading from File

**Issue:** `load_cookies` function in `http.rs` (lines 93-115) loads cookies from a JSON file without validation.

**Files affected:**
- `src-tauri/src/services/http.rs` (lines 93-115)

**Impact:**
- Cookies might be loaded from arbitrary paths if function is called incorrectly
- No validation of cookie content or expiry

**Fix approach:** Validate JSON structure before parsing, and ensure cookie domain matches expected source.

---

## Performance Concerns

### [HIGH] Unbounded Image Cache - Memory Leak Risk

**Issue:** `imageCache` in `Gallery.vue` (line 35) grows indefinitely with no size limit or eviction policy.

**Files affected:**
- `src/views/Gallery.vue` (lines 35, 48-58)

**Impact:**
- Memory usage grows continuously as user browses images
- Eventually causes browser tab crash on systems with limited memory
- Base64 encoding doubles memory usage for images

**Fix approach:** Implement LRU cache with maximum size (e.g., 50 images, ~100MB limit).

---

### [MEDIUM] Synchronous Directory Scanning

**Issue:** Directory scanning operations could block the async runtime for large directories.

**Files affected:**
- `src-tauri/src/commands/gallery.rs`: `get_directory_tree`, `get_directory_images`

**Impact:**
- UI may freeze for several seconds on directories with 10,000+ images
- `spawn_blocking` is used correctly but the blocking operation is expensive

**Fix approach:** Consider pagination for directory listing, or use streaming directory iteration.

---

### [MEDIUM] No Image Request Deduplication

**Issue:** Multiple concurrent requests for the same image can be triggered simultaneously.

**Files affected:**
- `src/views/Gallery.vue` (lines 48-58, 259-269)
- `src/views/Home.vue` (line 321)

**Impact:**
- Wasted bandwidth from duplicate downloads
- Race conditions when cache is updated during parallel requests

**Fix approach:** Implement request deduplication using in-flight request tracking.

---

### [LOW] Base64 Encoding Repeated on Cache Miss

**Issue:** When image load fails, base64 is fetched again even if it was recently attempted.

**Files affected:**
- `src/views/Gallery.vue` (lines 259-269)

**Impact:** Minor bandwidth waste on error recovery

**Fix approach:** Track failed requests temporarily to avoid immediate retries.

---

## Error Handling Gaps

### [HIGH] Settings Not Persisted

**Issue:** `useSettingsStore` uses reactive refs but never persists or loads settings.

**Files affected:**
- `src/stores/settings.ts`

**Impact:**
- All settings reset to defaults on app restart
- User's preferences (theme, download path, proxy settings) are lost

**Fix approach:** Integrate with Tauri store plugin or localStorage to persist settings.

---

### [MEDIUM] Silent Error Swallowing

**Issue:** Some errors are logged to console but not shown to user.

**Files affected:**
- `src/views/Home.vue` (lines 267-268, 324-328): `console.error` without user notification
- `src/views/Gallery.vue` (lines 55, 102-103, 135-136, 145-146): `console.error/warn` without UI feedback
- `src/stores/favoriteTags.ts` (line 16): Silent failure on loadTags

**Impact:**
- User unaware of failures
- Difficult to diagnose issues
- Poor user experience

**Fix approach:** Display errors using `useMessage()` or toast notifications.

---

### [MEDIUM] No Download Retry Logic

**Issue:** Failed downloads do not automatically retry.

**Files affected:**
- `src-tauri/src/commands/download.rs`: `start_download` function

**Impact:**
- Transient network errors cause permanent failure
- User must manually re-add failed downloads

**Fix approach:** Implement exponential backoff retry (3 attempts) for network failures.

---

### [LOW] No Loading States for Some Operations

**Issue:** Some async operations lack loading state management.

**Files affected:**
- `src/stores/favoriteTags.ts`: `loadTags`, `addParentTag`, `addChildTag`, `removeTag`
- `src/stores/gallery.ts`: Store state not connected to loading UI

**Impact:** User may click multiple times, not knowing operation is in progress

**Fix approach:** Expose loading state from stores and handle in components.

---

## Maintainability Concerns

### [MEDIUM] Missing TypeScript Types

**Issue:** Several TypeScript interfaces are missing or incomplete.

**Files affected:**
- `src/views/Gallery.vue`: Uses inline types `ImageInfo`, `SubDirInfo`, `TreeNode`
- `src/views/Home.vue`: Uses `PostMeta` inline

**Impact:**
- Code duplication
- Potential type inconsistencies
- Harder to maintain

**Fix approach:** Move all shared types to `src/types/` directory.

---

### [MEDIUM] Hardcoded Magic Numbers

**Issue:** Magic numbers scattered throughout code without named constants.

**Files affected:**
- `src-tauri/src/commands/download.rs` (line 77): `Semaphore::new(3)` - concurrency limit
- `src-tauri/src/services/http.rs` (line 31): `timeout(Duration::from_secs(60))`
- `src-tauri/src/commands/download.rs` (line 296): `downloaded % (100 * 1024)`
- `src/views/Gallery.vue` (line 475): `minmax(150px, 1fr)` - grid sizing

**Impact:**
- Unclear meaning of numbers
- Difficult to adjust values globally

**Fix approach:** Define named constants in appropriate modules.

---

### [LOW] Code Duplication in Rust Error Handling

**Issue:** Error formatting pattern is repeated multiple times.

**Files affected:**
- `src-tauri/src/commands/gelbooru.rs` (lines 36-40, 60-62)
- `src-tauri/src/commands/download.rs` (lines 216-218, 233-235, etc.)

**Impact:** Maintenance burden when error format changes

**Fix approach:** Create helper function for error formatting.

---

### [LOW] No Standardized Error Types

**Issue:** Rust backend uses `String` for errors instead of typed errors.

**Files affected:**
- All `#[tauri::command]` functions return `Result<T, String>`

**Impact:**
- Loss of error type information
- No compile-time checking of error variants

**Fix approach:** Define custom error enum with `thiserror` crate.

---

## Technical Debt

### [MEDIUM] No Test Coverage

**Issue:** No unit or integration tests found in the codebase.

**Impact:**
- High risk of regressions
- Fear of refactoring
- No documentation of expected behavior

**Fix approach:**
- Add Vitest for Vue component tests
- Add Rust unit tests with `#[cfg(test)]`
- Target 80% coverage for business logic

---

### [MEDIUM] Database Schema Not Versioned

**Issue:** SQLite database schema is created without version tracking or migrations.

**Files affected:**
- `src-tauri/src/db/mod.rs` (lines 38-78)

**Impact:**
- Cannot safely update schema in future versions
- User data may be lost on schema changes

**Fix approach:** Implement database migration system using rusqlite migrations or similar.

---

### [LOW] In-Memory Task State Not Persisted

**Issue:** Download tasks are stored in `DOWNLOAD_MANAGER` global state, lost on app restart.

**Files affected:**
- `src-tauri/src/commands/download.rs` (lines 11-14)

**Impact:**
- User loses track of pending downloads after restart
- No resume capability for partially downloaded files

**Fix approach:** Persist task state to database and resume incomplete downloads on startup.

---

### [LOW] Missing ESLint/Prettier Configuration

**Issue:** No linting or formatting configuration for frontend code.

**Impact:**
- Code style inconsistencies
- No automated quality checks

**Fix approach:** Add `.eslintrc.js` and `.prettierrc` with project-specific rules.

---

## Performance Bottlenecks

### [MEDIUM] Large Directory Tree Loading

**Issue:** `get_directory_tree` recursively scans entire directory structure on load.

**Files affected:**
- `src-tauri/src/commands/gallery.rs` (lines 138-227)

**Impact:** Slow startup for deep directory hierarchies

**Fix approach:** Lazy-load subdirectory contents on expand, not upfront.

---

### [LOW] Image Preloading Not Prioritized

**Issue:** `preloadImages` loads images sequentially without prioritization.

**Files affected:**
- `src/views/Gallery.vue` (lines 48-58)

**Impact:** Visible thumbnails may load slowly if followed by off-screen images

**Fix approach:** Use IntersectionObserver to preload only visible + buffer zone images.

---

## Fragile Areas

### [MEDIUM] Settings Store Race Condition

**Issue:** `useSettingsStore` is not initialized before being used by other stores.

**Files affected:**
- `src/stores/download.ts` (line 133): `const settingsStore = useSettingsStore()`

**Impact:** If `downloadStore` is used before `settingsStore` is mounted, behavior is undefined

**Fix approach:** Use Pinia plugin or initialize settings at app startup.

---

### [LOW] Route State Restoration Race Condition

**Issue:** `restorePageState` may conflict with `watch` callback in `Home.vue`.

**Files affected:**
- `src/views/Home.vue` (lines 452-471, 480-485)

**Impact:** Page may search twice on restoration due to `isRestoring.value` timing

**Fix approach:** Use `watchEffect` with proper dependency tracking or queue the search.

---

## Dependencies at Risk

### [LOW] `scraper` Crate Stability

**Issue:** HTML parsing depends on `scraper` crate version 0.22.

**Files affected:**
- `src-tauri/Cargo.toml`

**Impact:** Gelbooru HTML changes could break parsing without warning

**Fix approach:** Add parsing tests and monitor for parsing failures.

---

## Summary by Severity

| Severity | Count | Key Issues |
|----------|-------|------------|
| HIGH | 2 | Unbounded cache, Settings not persisted |
| MEDIUM | 12 | Path sanitization, No tests, Error handling gaps, Path traversal risks |
| LOW | 11 | Missing configs, Code duplication, Minor optimizations |

---

*Concerns audit: 2026-04-13*
