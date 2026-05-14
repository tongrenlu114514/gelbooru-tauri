# Phase 9: Download Retry UI - Context

**Gathered:** 2026-05-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Improve the Download manager UI with one-click retry, visible pause/resume controls, and clear error messaging for failed downloads. Backend already has pause/resume/retry implemented — this phase focuses on UI polish and UX completion.

</domain>

<decisions>
## Implementation Decisions

### Error display format
- Error message shown inline in the task row (not just tooltip)
- Error badge with red background for failed tasks
- Show HTTP status code if available (e.g., "404", "500")
- Truncate long error messages at ~80 chars with expand on click

### Retry UX
- Dedicated "Retry" button in failed task row (prominent, not buried in menu)
- Icon: refresh/retry icon from naive-ui icon set
- Button text: "Retry" with optional icon
- Retry resets progress bar to 0% immediately (no confusing partial state)

### Pause/Resume UX
- Single toggle button that switches between pause/resume icon
- Show "Paused" status badge for paused tasks
- Paused tasks remain in the downloading section with reduced visual emphasis

### Visual hierarchy
- Failed tasks: highlighted row with error color, prominent retry button
- Active downloads: normal emphasis with progress bar
- Paused tasks: muted/gray treatment, resume button visible
- Completed tasks: success color, clear action buttons

### Action button placement
- All per-row actions visible inline (no hover menus)
- Standard actions: Pause/Resume, Open, Delete
- Failed-only action: Retry
- Compact row design — avoid expanding row height for failed tasks

### Stats bar
- Show failed task count prominently (e.g., "3 failed" in red)
- Failed count links to filtering to failed tasks only

### Claude's Discretion
- Exact color tokens for error/success/warning states
- Animation style for retry action (spinner vs bounce)
- Whether to auto-retry after certain errors (network timeout vs 404)
- Toast/notification behavior on retry success/failure

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Download management
- `src/stores/download.ts` — DownloadTask type, Pinia store structure, existing actions
- `src/views/Downloads.vue` — Current download manager UI, existing components
- `src-tauri/src/commands/download.rs` — Rust backend commands (pause_download, resume_download, start_download)

### Design system
- No formal design spec — use naive-ui component library consistently
- Existing color tokens from naive-ui theme

</canonical_refs>

<codebase_context>
## Existing Code Insights

### Reusable Assets
- `DownloadNotifier.vue` — Notification component for success/error notifications
- naive-ui `NButton`, `NBadge`, `NProgress`, `NTag` — all used in existing UI
- Existing status badge pattern with color-coded NTag

### Established Patterns
- Pinia store drives all download state — new UI hooks into existing store
- Backend emits `download-progress` events — UI subscribes via `initListeners()`
- Tauri commands: `pause_download(id)`, `resume_download(app, db, id)`, `start_download(app, db, id)`

### Integration Points
- `Downloads.vue` — main download manager view, all new UI goes here
- `download.ts` store — add any new computed properties or actions here
- Task row actions in the `NDataTable` row-actions column
</codebase_context>

<specifics>
## Specific Ideas

- Failed downloads should be immediately actionable — not buried
- Users should see WHY a download failed without clicking into details
- Retry should feel instant (button press → immediate feedback)
- "3 failed" in the stats bar should act as a filter to show only failed tasks

</specifics>

<deferred>
## Deferred Ideas

- Auto-retry with exponential backoff — backend already has retry logic, but no UI to configure
- Batch retry all failed tasks — nice-to-have, out of scope for this phase
- Download speed throttling UI — separate phase
- Per-task priority/ordering — separate phase

</deferred>

---

*Phase: 09-download-retry-ui*
*Context gathered: 2026-05-15*