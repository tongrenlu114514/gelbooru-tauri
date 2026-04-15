---
phase: 03
slug: performance-reliability
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-16
---

# Phase 03 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Vitest (frontend) + `#[test]` / rstest (Rust) |
| **Config file** | `vitest.config.ts` (frontend), `Cargo.toml` [dev-dependencies] (Rust) |
| **Quick run command (frontend)** | `pnpm vitest run src/views/Gallery.spec.ts` |
| **Quick run command (Rust)** | `cargo test --lib -- commands::gallery commands::download -- --test-threads=1` |
| **Full suite command (frontend)** | `pnpm vitest run` |
| **Full suite command (Rust)** | `cargo test --lib` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run affected test file only (`cargo test --lib commands::<module>` or `pnpm vitest run src/views/Gallery.spec.ts`)
- **After every plan wave:** Full suite (`cargo test --lib && pnpm vitest run`)
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 03-01-01 | 01 | 1 | REQ-3.1 | — | N/A | unit (Vue) | `pnpm vitest run src/views/Gallery.spec.ts` | W0 | ⬜ pending |
| 03-01-02 | 01 | 1 | REQ-3.2 | — | N/A | unit (Vue) | `pnpm vitest run src/views/Gallery.spec.ts` | W0 | ⬜ pending |
| 03-02-01 | 02 | 1 | REQ-3.3 | T-03-01 | No infinite retry loop | unit (Rust) | `cargo test download_with_retry` | W0 | ⬜ pending |
| 03-02-02 | 02 | 1 | REQ-3.4 | T-03-01 | No retry on 4xx | unit (Rust) | `cargo test download_with_retry` | W0 | ⬜ pending |
| 03-02-03 | 02 | 1 | REQ-3.5 | — | Cancellation respected | unit (Rust) | `cargo test download_with_retry` | W0 | ⬜ pending |
| 03-03-01 | 03 | 1 | REQ-3.6 | — | N/A | unit (Rust) | `cargo test build_tree` | W0 | ⬜ pending |
| 03-03-02 | 03 | 1 | REQ-3.7 | — | N/A | unit (Rust) | `cargo test build_tree` | W0 | ⬜ pending |
| 03-04-01 | 04 | 1 | REQ-3.8 | T-03-02 | Rate limit enforced globally | unit (Rust) | `cargo test rate_limit` | W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/views/Gallery.spec.ts` — test IntersectionObserver lazy loading behavior (REQ-3.1, 3.2)
- [ ] `src-tauri/src/commands/download.rs` — add unit tests for retry logic (REQ-3.3, 3.4, 3.5)
- [ ] `src-tauri/src/commands/gallery.rs` — add unit tests for async directory scan (REQ-3.6, 3.7)
- [ ] `src-tauri/src/services/http.rs` — add unit tests for rate limiting (REQ-3.8)

*Phase 3 is purely code changes with no new external dependencies. Vitest and Rust test infrastructure already configured from Phase 2.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Gallery scroll performance (memory) | REQ-3.1 | Memory leak only visible under load with heap profiling | Open DevTools → Performance tab → scroll gallery with 100+ images → check JS heap for growth |
| Large directory scan speed | REQ-3.6 | Requires real filesystem with >1000 files | Run against test directory, measure elapsed time |

---

## Security Threat Register

| Threat ID | Category | Component | Disposition | Mitigation |
|-----------|----------|-----------|-------------|------------|
| T-03-01 | Denial of Service | download.rs | mitigate | Hard cap of 3 retries (per D-04) — exponential backoff 1s→2s→4s |
| T-03-02 | Denial of Service | http.rs | mitigate | Global per-client rate limiter (per D-10), not per-task |
| T-03-03 | Information Disclosure | gallery.rs | accept | Path traversal already mitigated by `validate_path_within_base` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending

---

*Phase: 03-performance-reliability*
*Validation strategy generated: 2026-04-16 from RESEARCH.md*
