---
phase: 4
slug: polish-release
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-17
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Frontend Framework** | Vitest |
| **Frontend Config** | `vitest.config.ts` |
| **Frontend Quick Run** | `pnpm test` |
| **Frontend Full Suite** | `pnpm vitest run` |
| **Backend Framework** | Rust built-in `cargo test` |
| **Backend Quick Run** | `cargo test` |
| **Backend Full Suite** | `cargo test --release` |
| **Estimated runtime** | ~30 seconds (full suite) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test` (backend) + `pnpm test` (frontend)
- **After every plan wave:** Run `cargo test --release && pnpm vitest run`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 4.1-01 | 01 | 1 | REQ-4.1 Schema versioning | unit | `cargo test test_schema_version` | ✅ db/mod.rs | ⬜ pending |
| 4.2-01 | 01 | 1 | REQ-4.2 Error consistency | unit | `cargo test` (existing tests green) | ✅ commands/*.rs | ⬜ pending |
| 4.3-01 | 02 | 2 | REQ-4.3 README created | file exists | `test -f README.md` | ✅ README.md | ⬜ pending |
| 4.4-01 | 02 | 2 | REQ-4.4 tauri.conf.json verified | file exists | `grep '"version": "1.0.0"' tauri.conf.json` | ✅ tauri.conf.json | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/src/db/mod.rs` — add `test_schema_version_baseline_for_existing_db` (verifies INSERT OR IGNORE sets version=1 for DBs without schema_version row)
- [ ] `src-tauri/src/db/mod.rs` — add `test_schema_version_runs_migrations_in_order` (verifies sequential version incrementing)

*Phase 2 already installed vitest + cargo test infrastructure. No new frameworks needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| NSIS installer produces .exe | REQ-4.4 Release build | Requires full `pnpm tauri build` + Windows VM | Run `pnpm tauri build`, verify `src-tauri/target/release/bundle/nsis/Gelbooru Downloader 1.0.0.exe` exists |
| Schema version table created on first run | REQ-4.1 DB migration | Requires first-run of new app binary | Launch app with fresh DB, query `SELECT version FROM schema_version` → expect `1` |
| NSIS installer installs correctly | REQ-4.4 | Windows installer behavior | Run .exe installer on clean Windows VM, verify app launches |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
