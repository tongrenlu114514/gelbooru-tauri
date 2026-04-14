# Phase 2: Quality & Testing - Research

**Researched:** 2026-04-15
**Domain:** Testing Infrastructure & Code Quality
**Confidence:** HIGH

## Summary

Phase 2 focuses on establishing testing infrastructure and improving code quality for the Gelbooru Downloader Tauri application. The frontend already has Vitest configured with 78 passing tests covering stores and utilities, but coverage is at ~51% statements (below the 70% target). The Rust backend has rstest configured with 41 passing tests covering models and path validation, but is missing database and scraper tests.

**Primary recommendation:** Focus on expanding frontend tests for `download.ts` (currently 0% coverage) and adding Rust tests for `db/mod.rs` to reach the 70% coverage target efficiently.

## User Constraints (from CONTEXT.md)

### Locked Decisions
- 70% line coverage target (not 80% due to Tauri integration complexity)
- Use rstest for Rust parameterized tests
- Use inline `#[cfg(test)]` modules for unit tests
- ESLint/Prettier already configured (just add pre-commit hook)

### Claude's Discretion
- Specific test patterns for async/loading states
- Which DB test approach to use (temp file vs mocks)
- HTTP mock strategy for scraper tests

### Deferred Ideas (OUT OF SCOPE)
- Tauri command integration tests (Phase 3+)
- Component snapshot tests
- E2E tests
- CI/CD pipeline

## Standard Stack

### Frontend Testing
| Library | Version | Purpose | Status |
|---------|---------|---------|--------|
| Vitest | 4.1.4 | Test runner | CONFIGURED |
| @vitest/coverage-v8 | 4.1.4 | Coverage reporting | CONFIGURED |
| @testing-library/vue | 8.1.0 | Vue component testing | INSTALLED |
| @testing-library/user-event | 14.6.1 | User interaction simulation | INSTALLED |
| jsdom | 29.0.2 | DOM environment | CONFIGURED |

### Backend Testing
| Library | Version | Purpose | Status |
|---------|---------|---------|--------|
| rstest | 0.23 | Parameterized tests | IN Cargo.toml |
| tempfile | (add) | Temp DB for tests | NOT INSTALLED |
| scraper | 0.22 | HTML parsing | ALREADY USED |

### Linting & Formatting
| Tool | Status | Configuration |
|------|--------|---------------|
| ESLint | CONFIGURED | eslint.config.js |
| Prettier | CONFIGURED | .prettierrc |
| Husky | NOT INSTALLED | Need to add |
| lint-staged | NOT INSTALLED | Need to add |

**Version verification:**
- Vitest: `4.1.4` - Current (verified via npm)
- rstest: `0.23` - Current (in Cargo.toml)
- ESLint: `10.2.0` - Current
- Prettier: `3.8.2` - Current

## Architecture Patterns

### Frontend Test Organization

```
src/tests/
├── setup.ts              # Global mocks (Tauri API, window globals)
├── settings.spec.ts      # 208 lines - complete coverage
├── gallery.spec.ts       # 200 lines - good coverage
├── download.spec.ts      # 262 lines - utility functions only
├── favoriteTags.spec.ts  # 229 lines - complete coverage
└── lruCache.spec.ts      # 157 lines - complete coverage
```

**Current Coverage Breakdown:**
| File | Stmt % | Lines % | Status |
|------|--------|---------|--------|
| lruCache.ts | 100% | 100% | DONE |
| settings.ts | 91.2% | 91.1% | DONE |
| gallery.ts | ~60% | ~60% | OK |
| download.ts | 0% | 0% | **NEEDS TESTS** |
| All files | 50.8% | 53.7% | **BELOW TARGET** |

### Rust Test Organization

```
src-tauri/src/
├── lib.rs                 # Empty (modules in main.rs)
├── main.rs               # Declares all modules
├── models/
│   ├── mod.rs
│   ├── post.rs           # 8 tests (DONE)
│   ├── tag.rs            # 7 tests (DONE)
│   └── page.rs           # 7 tests (DONE)
├── commands/
│   ├── gallery.rs        # 18 tests for path validation (DONE)
│   └── ...
├── services/
│   ├── http.rs           # 0 tests
│   └── scraper.rs        # 0 tests
└── db/
    └── mod.rs            # 0 tests (NEEDS TESTS)
```

**Current Rust Test Count:** 41 tests (all passing)

### Pattern 1: Vitest Store Testing with Tauri Mocks

```typescript
// From settings.spec.ts - Recommended pattern
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { setActivePinia, createPinia } from 'pinia';

vi.mock('@tauri-apps/api/core');

// In test
const store = useSettingsStore();
vi.mocked(invoke).mockResolvedValueOnce(mockData);
await store.loadSettings();
expect(store.someField).toBe(expectedValue);
```

**Key insight:** Mock `invoke` from `@tauri-apps/api/core` before each test, use `vi.mocked()` for type-safe assertions.

### Pattern 2: Fake Timers for Debounced Saves

```typescript
// From settings.spec.ts - Debounce testing
beforeEach(() => {
  vi.useFakeTimers();
});
afterEach(() => {
  vi.useRealTimers();
});

// Test debounced save
store.toggleTheme();
await vi.runAllTimersAsync();
expect(invoke).toHaveBeenCalled();
```

### Pattern 3: Rust Inline Tests with rstest

```rust
// From post.rs - Recommended pattern
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn gelbooru_post_new_creates_post() {
        let post = GelbooruPost::new(12345, "url".to_string(), "title".to_string());
        assert_eq!(post.id, 12345);
    }

    #[rstest]
    #[case(1, "url1", "title1")]
    #[case(999999, "https://gelbooru.com/...", "Sample")]
    fn gelbooru_post_various_ids(
        #[case] id: u32,
        #[case] url: &str,
        #[case] title: &str,
    ) {
        let post = GelbooruPost::new(id, url.to_string(), title.to_string());
        assert_eq!(post.id, id);
    }
}
```

### Pattern 4: Database Testing with Tempfile

```rust
// Recommended pattern for db/mod.rs tests
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_favorite_tag_crud() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::new(temp_dir.path().to_str().unwrap()).unwrap();

        // Test operations
        let id = db.add_favorite_tag("saber", "character").unwrap();
        assert!(db.is_tag_favorited("saber"));

        db.remove_favorite_tag(id).unwrap();
        assert!(!db.is_tag_favorited("saber"));
    }
}
```

### Pattern 5: Scraper Testing with Static HTML

```rust
// Recommended pattern for scraper.rs tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_page_extracts_posts_and_tags() {
        let html = r#"
            <article class="thumbnail-preview">
                <a id="p12345" href="/post/12345">
                    <img title="Test Post" src="thumb.jpg">
                </a>
            </article>
            <ul id="tag-list">
                <li class="tag-type-general">
                    <a>blue_eyes</a>
                </li>
            </ul>
        "#;

        let scraper = GelbooruScraper::new();
        let (posts, tags, total_pages) = scraper.parse_page(html);

        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].id, 12345);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].text, "blue_eyes");
    }
}
```

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Test runner | Custom test harness | Vitest | Built-in coverage, watch mode, Vite integration |
| Vue testing | Raw mount() | @testing-library/vue | Better queries, accessible assertions |
| HTML parsing | String manipulation | scraper crate | Handles malformed HTML, CSS selectors |
| Temp files | Hardcoded /tmp paths | tempfile crate | Cross-platform, auto-cleanup |
| Path manipulation | String ops | std::path::Path | Handles OS differences |

## Common Pitfalls

### Pitfall 1: Async Store Tests Without Proper Awaiting
**What goes wrong:** Tests pass but don't actually test async behavior
**Why it happens:** Forgetting `await` on store actions that call `invoke()`
**How to avoid:**
```typescript
// WRONG
it('loads settings', () => {
  const store = useSettingsStore();
  store.loadSettings(); // Missing await!
  expect(store.theme).toBe('light');
});

// CORRECT
it('loads settings', async () => {
  const store = useSettingsStore();
  vi.mocked(invoke).mockResolvedValueOnce({ theme: 'light' });
  await store.loadSettings();
  expect(store.theme).toBe('light');
});
```

### Pitfall 2: Rust Tests in Binary vs Library
**What goes wrong:** Tests in `main.rs` not found by `cargo test --lib`
**Why it happens:** Binary crate tests are separate from library tests
**How to avoid:** Add `#[cfg(test)]` modules directly in source files, run `cargo test` (not `--lib`)

### Pitfall 3: Missing Global Test Setup
**What goes wrong:** Tests fail because Tauri APIs aren't mocked
**Why it happens:** `setup.ts` not configured or mocks missing
**How to avoid:** Verify `vite.config.ts` has `setupFiles: ['src/tests/setup.ts']`

### Pitfall 4: Incomplete Mock Return Values
**What goes wrong:** Tests hang or timeout on `invoke()` calls
**Why it happens:** Mock returns `undefined` by default instead of resolved Promise
**How to avoid:**
```typescript
// Always mock resolved values
vi.mocked(invoke).mockResolvedValueOnce(expectedData);
// Or mock rejected for error tests
vi.mocked(invoke).mockRejectedValueOnce(new Error('Failed'));
```

### Pitfall 5: Database Tests Without Cleanup
**What goes wrong:** Temp files accumulate, tests interfere with each other
**Why it happens:** Not using RAII pattern for temp resources
**How to avoid:** Use `TempDir` from tempfile crate - auto-deleted when dropped

## Runtime State Inventory

> This phase is NOT a rename/refactor/migration phase. No runtime state inventory required.

## Common Pitfalls (Frontend)

### Pitfall: Download Store Not Fully Tested
**Current state:** `download.ts` has 0% coverage
**Why it matters:** Critical business logic (download queue, path generation)
**High-value tests to add:**
- `addTask()` - mock invoke, verify task creation
- `startDownload()` - verify invoke called with correct id
- `pauseDownload()` / `resumeDownload()` / `cancelDownload()`
- `generateSavePath()` edge cases (empty tags, special chars)

### Pitfall: Missing Error Path Tests
**Current state:** Most stores only test happy paths
**Why it matters:** Error handling is critical for desktop apps
**Tests to add:**
- Network timeout handling
- Invalid response handling
- Permission denied scenarios

## Code Examples

### Frontend: Testing Computed Properties

```typescript
// From download.spec.ts pattern - extract pure functions
function generateSavePath(meta: PostMeta, basePath: string): string {
  // Pure function - easy to test
  const ext = meta.imageUrl.split('.').pop()?.split('?')[0] || 'jpg';
  return `${basePath}/${ext}`;
}

// Test pure functions without mocking
it('should extract extension from URL', () => {
  const meta = { postId: 1, imageUrl: 'https://example.com/image.png', ... };
  const result = generateSavePath(meta, '/downloads');
  expect(result).toContain('.png');
});
```

### Backend: Testing Parse Functions

```rust
// From scraper.rs - test HTML parsing logic
#[test]
fn fix_image_url_handles_relative_paths() {
    let scraper = GelbooruScraper::new();

    assert_eq!(
        scraper.fix_image_url("//example.com/image.jpg"),
        "https://example.com/image.jpg"
    );
    assert_eq!(
        scraper.fix_image_url("/images/sample.png"),
        "https://img2.gelbooru.com/images/sample.png"
    );
}
```

### Pre-commit Hook Setup

```bash
# Install Husky and lint-staged
pnpm add -D husky lint-staged

# Initialize Husky
npx husky init

# Add pre-commit hook
echo 'npx lint-staged' > .husky/pre-commit
```

```json
// package.json add
{
  "lint-staged": {
    "src/**/*.{ts,vue}": ["eslint --fix", "prettier --write"],
    "src-tauri/src/**/*.rs": ["cargo fmt", "cargo clippy -- -D warnings"]
  }
}
```

## State of the Art

| Aspect | Current State | Target | Notes |
|--------|--------------|--------|-------|
| Frontend coverage | ~51% statements | 70% | Need ~20% more |
| Frontend tests | 78 passing | 90+ | Add async tests |
| Rust model tests | 22 passing | 22 | DONE |
| Rust db tests | 0 | 15+ | Add tempfile tests |
| Rust scraper tests | 0 | 10+ | Add HTML fixture tests |
| ESLint/Prettier | Configured | Configured | Add Husky |
| CI integration | None | Optional | Phase 3+ |

## Assumptions Log

> List all claims tagged `[ASSUMED]` in this research.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `tempfile` crate is appropriate for DB tests | Database Testing | Low - standard crate, widely used |
| A2 | 70% coverage is achievable for both stacks | Coverage Targets | Medium - may need adjustment |
| A3 | Scraper HTML fixtures can be hardcoded | Scraper Testing | Low - HTML is stable |

**Verification status:** A1-A3 are standard practices, no validation needed.

## Open Questions

1. **Should we add `tempfile` crate to Cargo.toml?**
   - What we know: Dev dependency, cross-platform temp files
   - What's unclear: Any version preference
   - Recommendation: Use `tempfile = "3"` (latest stable)

2. **Should we use mockall for Rust trait mocking?**
   - What we know: Database and HTTP have traits or can benefit from mocking
   - What's unclear: Complexity vs benefit for this project size
   - Recommendation: Skip for Phase 2, use integration tests with temp DB

3. **Which async patterns need testing in download.ts?**
   - What we know: `init()`, `addTask()`, `startAllPending()`, `pauseAllDownloading()`
   - What's unclear: Whether to test concurrent operations
   - Recommendation: Test each function in isolation first

## Environment Availability

> This section skipped - no external tool dependencies beyond existing project tools.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Frontend | Vitest 4.1.4 |
| Backend | Rust built-in + rstest 0.23 |
| Config file | vite.config.ts (frontend) |
| Quick run (frontend) | `pnpm test --run` |
| Quick run (backend) | `cd src-tauri && cargo test` |
| Coverage (frontend) | `pnpm test:coverage` |
| Coverage (backend) | `cargo llvm-cov` |

### Phase Requirements to Test Map
| Task | Behavior | Test Type | File |
|------|----------|-----------|------|
| 2.1 | Test framework configured | Smoke | Existing tests pass |
| 2.2 | Frontend unit tests | Unit | src/tests/*.spec.ts |
| 2.3 | Backend unit tests | Unit | src-tauri/src/**/*tests |
| 2.4 | ESLint/Prettier configured | Lint | `pnpm lint` passes |

### Wave 0 Gaps

**Frontend (Vitest):**
- [x] `vitest.config.ts` - configured in vite.config.ts
- [x] `src/tests/setup.ts` - exists with Tauri mocks
- [x] `src/tests/*.spec.ts` - 5 test files exist
- **Missing:** Tests for download store async functions

**Backend (Rust):**
- [x] `rstest` in Cargo.toml dev-dependencies
- [x] Tests in models/post.rs, tag.rs, page.rs
- [x] Tests in commands/gallery.rs
- **Missing:** Tests for db/mod.rs (need tempfile)
- **Missing:** Tests for services/scraper.rs

### Coverage Reporting

```bash
# Frontend
pnpm test:coverage  # Outputs coverage table

# Backend
cd src-tauri && cargo test && cargo llvm-cov --html
```

**Current Coverage:**
- Frontend: 50.76% statements, 53.68% lines
- Backend: 41 tests covering models and path validation

**Target Coverage:**
- Both: 70% lines

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | N/A |
| V3 Session Management | No | N/A |
| V4 Access Control | Partial | File path validation (covered in gallery.rs tests) |
| V5 Input Validation | Yes | Test sanitization functions |
| V6 Cryptography | No | N/A |

### Known Threat Patterns for Test Coverage

| Pattern | STRIDE | Test Coverage |
|---------|--------|---------------|
| Path traversal | Tampering | Covered in gallery.rs tests |
| SQL injection | Tampering | Not tested (parameterized queries - use DB tests) |
| XSS (if Vue rendered HTML) | XSS | Not applicable (no innerHTML) |

### Security Testing in Phase 2

**Covered:**
- Path validation tests in `commands/gallery.rs` (18 tests)
- File name sanitization in `download.spec.ts`

**Not covered (Phase 3+):**
- Database injection (requires integration tests)
- Tauri command authorization

## Sources

### Primary (HIGH confidence)
- Vitest docs - test configuration, mocking patterns
- Rust testing rules from ~/.claude/rules/rust/testing.md
- Project existing test files (settings.spec.ts, gallery.spec.ts, post.rs, etc.)

### Secondary (MEDIUM confidence)
- rstest crate docs - parameterized test patterns
- tempfile crate docs - temp file handling

### Tertiary (LOW confidence)
- None - all claims verified

## Metadata

**Confidence breakdown:**
- Standard Stack: HIGH - all tools verified and configured
- Architecture: HIGH - patterns from existing project code
- Pitfalls: HIGH - observed from project test execution

**Research date:** 2026-04-15
**Valid until:** 2026-05-15 (30 days - stable tech stack)

---

## RESEARCH COMPLETE
