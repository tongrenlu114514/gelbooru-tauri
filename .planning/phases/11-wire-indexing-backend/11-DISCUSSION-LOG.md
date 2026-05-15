# Phase 11: Wire Indexing Backend - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-16
**Phase:** 11-wire-indexing-backend
**Areas discussed:** Setup Failure Handling, Command Registration Style

---

## Setup Failure Handling

| Option | Description | Selected |
|--------|-------------|----------|
| A) Abort | `.expect()` on setup, app exits on failure | |
| B) Graceful Degrade | Log error, app continues, thumbnails fallback to sync | ✓ |

**User's choice:** B — Graceful Degrade

**Notes:**
- IndexingService failure should not block core features (download, search)
- Background queue unavailable → on-demand synchronous path still works
- Error logged to stderr, no blocking dialog

---

## Command Registration Style

| Option | Description | Selected |
|--------|-------------|----------|
| 1) Single block | All commands in one flat list | |
| 2) Module-commented | Grouped by module with `//` comments | ✓ |

**User's choice:** 2 — Module-commented style

**Notes:**
- Matches existing `main.rs` style (gelbooru, download, gallery blocks)
- Easier to locate which phase added which commands

---

## Claude's Discretion

All remaining decisions (exact line placement, error log format, etc.) left to planner/implementer.

---

*Discussion complete: 2026-05-16*