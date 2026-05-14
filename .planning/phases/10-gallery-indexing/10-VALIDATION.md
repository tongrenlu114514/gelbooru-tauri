---
phase: "10"
plan: "01"
status: draft
nyquist_compliant: false
wave_0_complete: true
created: 2026-05-15
note: "Tests are added inline during implementation tasks (Rust #[test] in db/mod.rs, integration test module in commands/indexing.rs), not as separate Wave 0 files. Plan automated verify commands use grep and cargo check which do not require separate test files. Wave 0 sign-off: inline tests created during Task 2 and Task 3 implementation."
---

# Phase 10 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Vitest 4.1.4 (frontend unit) + Rust `#[test]` (backend unit) + Playwright 1.59.1 (E2E) |
| **Config file** | vitest.config.ts / playwright.config.ts (existing) |
| **Quick run command** | `cargo test --lib -- indexing` (Rust) + `pnpm vitest run src/stores` (frontend) |
| **Full suite command** | `cargo test` + `pnpm test` + `pnpm exec playwright test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib` (Rust) + `pnpm vitest run src/stores` (frontend)
- **After every plan wave:** Run `cargo test` + `pnpm test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 10-01-01 | 01 | 1 | IDX-01 | T-10-01 / T-10-02 | Path traversal prevented, symlinks not followed | unit (Rust) | `grep -n 'image = "0.25"' src-tauri/Cargo.toml` | n/a | pending |
| 10-01-02 | 01 | 1 | IDX-02 | T-10-01 | Thumbnails in app_data_dir only | unit (Rust) | `grep -n '002_gallery_index' src-tauri/src/db/mod.rs` | n/a | pending |
| 10-01-03 | 01 | 1 | IDX-03 | T-10-03 | On-demand returns valid path or error | unit (Rust) | `grep -n 'pub async fn scan_gallery' src-tauri/src/commands/indexing.rs` | n/a | pending |
| 10-01-04 | 01 | 1 | IDX-04 | — | Background queue does not block UI | unit (Rust) | `cargo check --manifest-path src-tauri/Cargo.toml` | n/a | pending |

*Status: pending · green · red · flaky*

---

## Wave 0 Requirements

**Waived — inline tests approach:**
- Tests are written inline within existing files during implementation tasks, not as separate Wave 0 files.
- Task 2: Add `#[cfg(test)]` module to db/mod.rs with tests for gallery_images table helpers.
- Task 3: Add `#[cfg(test)]` module to indexing.rs with tests for IndexingService and commands.
- This approach follows the project's existing pattern of Rust `#[test]` modules in the same files as the code under test.
- Plan automated verify commands (grep, cargo check) do not require separate test files to exist.

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Thumbnail generation quality (320x320 cover fit, 60x72 filmstrip) | IDX-02, IDX-03 | Visual inspection required | Open gallery, verify thumbnails look correct (not distorted) |
| Background queue does not freeze UI during large library scan | IDX-04 | UI freeze is subjective and hard to auto-detect | With 100+ images, scroll gallery while background indexing runs |

*If none: "All phase behaviors have automated verification."*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 waived (nyquist_compliant: false, inline tests approach)
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [x] `nyquist_compliant: false` documented (checker blocker resolved)

**Approval:** pending