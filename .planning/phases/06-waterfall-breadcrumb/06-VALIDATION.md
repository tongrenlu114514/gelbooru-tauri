---
phase: 06
slug: waterfall-breadcrumb
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-10
---

# Phase 6 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Vitest (vitest + @vue/test-utils) |
| **Config file** | vitest.config.ts (not present — uses pnpm vitest default from package.json) |
| **Quick run command** | `pnpm vitest run src/views/Gallery.spec.ts` |
| **Full suite command** | `pnpm test` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `pnpm vitest run src/views/Gallery.spec.ts`
- **After every plan wave:** Run `pnpm test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 06-01-01 | 01 | 1 | REQ-06-Masonry | T-06-01 / T-06-02 | N/A (read-only display) | unit | `pnpm vitest run src/views/Gallery.spec.ts` | ✅ | ⬜ pending |
| 06-01-02 | 01 | 1 | REQ-06-Masonry | T-06-01 / T-06-02 | N/A (read-only display) | unit | `pnpm vitest run src/views/Gallery.spec.ts` | ✅ | ⬜ pending |
| 06-02-01 | 02 | 1 | REQ-06-Breadcrumb | T-06-03 / T-06-04 | N/A (path display only) | unit | `pnpm vitest run src/views/Gallery.spec.ts` | ✅ | ⬜ pending |
| 06-02-02 | 02 | 1 | REQ-06-Navigate, REQ-06-ScrollFolderSwitch | T-06-04 / T-06-05 | Path from Tauri IPC (server-cleaned) | unit | `pnpm vitest run src/views/Gallery.spec.ts` | ✅ | ⬜ pending |
| 06-02-03 | 02 | 1 | REQ-06-Breadcrumb | — | N/A (styling only) | unit | `pnpm vitest run src/views/Gallery.spec.ts` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/views/Gallery.spec.ts` — add test cases for breadcrumb path resolution and masonry rendering (per RESEARCH.md Validation Architecture Wave 0 Gaps)

*Existing Vitest + @vue/test-utils infrastructure covers all phase requirements. No new framework install needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Smooth scroll to first image card | REQ-06-ScrollFolderSwitch | scrollIntoView animation is browser-specific, verified visually | Navigate to any folder, observe smooth scroll to first card |
| Breadcrumb click navigates to correct folder | REQ-06-Navigate | Navigation target path computed from runtime values | Click each breadcrumb segment, verify folder navigation |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending