---
phase: 2
phase_name: Quality & Testing
status: discussed
last_updated: "2026-04-15"
---

# Phase 2 Context

## Phase Scope

From ROADMAP.md, Phase 2 includes:
- 2.1: 配置测试框架 (Vitest + Rust tests) - **MOSTLY DONE**
- 2.2: 单元测试 (前端) - **NEEDS EXPANSION**
- 2.3: 单元测试 (后端) - **NEEDS CREATION**
- 2.4: 配置 ESLint/Prettier - **DONE**

## Prior Context (from Phase 1)

- Phase 1 completed 4/4 tasks
- Settings persistence to database added
- Download task persistence added
- Path validation utilities added
- Hardcoded paths removed

## Gray Areas & Decisions

### 1. Test Coverage Targets

**Decision:** 70% line coverage target for this phase

Rationale:
- Frontend: 4 test files exist (settings, gallery, download, favoriteTags, lruCache) - estimate ~60-70% coverage
- Backend: No tests yet - focus on high-value modules first
- 80% is ideal but may require extensive mocking setup for Tauri integration

### 2. Frontend Test Files to Add/Improve

**Decision:** Add tests for these areas:

Priority (high to low):
1. `stores/gallery.ts` - Add async/loading state tests
2. `stores/download.ts` - Add download queue management tests
3. `stores/favoriteTags.ts` - Already has tests, verify coverage
4. Utility functions in `utils/` - If any exist
5. Components - Visual/regression tests for key views (optional)

### 3. Backend Rust Modules to Test

**Decision:** Focus on testable modules first:

Priority (high to low):
1. `models/*.rs` - Model parsing and validation (easy to unit test)
2. `db/mod.rs` - Database operations (can use temp DB or mocks)
3. `services/scraper.rs` - HTML parsing logic (can mock HTTP responses)
4. `services/http.rs` - HTTP client (need careful mocking setup)
5. `commands/*.rs` - Tauri commands (skip for now, harder to test)

### 4. Rust Testing Approach

**Decision:** Use rstest for parameterized tests, inline tests for simple cases

Pattern:
- Use `#[cfg(test)]` modules in each source file
- Use rstest for test cases with multiple inputs
- Create a `tests/` directory for integration-style tests
- Use tempfile for database tests
- Mock HTTP responses using `reqwest-mock` or similar pattern

### 5. ESLint/Prettier Status

**Decision:** Configuration is complete, just verify and add pre-commit hook

- ESLint config exists: `eslint.config.js`
- Prettier config exists: `.prettierrc`
- Add Husky pre-commit hook to run lint + format checks

## Implementation Boundaries

### What IS in scope:
- Achieve 70% coverage for new tests
- Add Rust unit tests for models and db module
- Verify ESLint/Prettier configuration
- Add pre-commit lint hook

### What is NOT in scope:
- Tauri command integration tests (defer to Phase 3 or 4)
- Component snapshot tests
- E2E tests (future phase)
- CI/CD pipeline setup

## Dependencies

- Frontend: Already configured (Vitest, @vitest/coverage-v8)
- Backend: rstest already in Cargo.toml dev-dependencies
- Missing: May need reqwest mock utilities for HTTP testing

## Notes

- Phase 1 established that database operations work correctly
- Phase 3 will address memory leaks (Gallery.vue imageCache)
- Tests should follow AAA pattern (Arrange-Act-Assert)
- Use descriptive test names that explain the scenario
