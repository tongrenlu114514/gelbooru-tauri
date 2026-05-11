# Research Summary: Gelbooru v1.2 - Image Viewer & Gallery Indexing

**Date:** 2026-05-12
**Status:** Research Complete

---

## Executive Summary

Gelbooru v1.2 adds image viewing and gallery management capabilities to the existing Tauri + Vue 3 desktop application. The core feature is a full-screen image viewer with zoom/pan controls, supplemented by tag autocomplete for search, download retry UI, and gallery indexing with thumbnail generation.

**Technology additions are minimal:** The only new dependency is the Rust `image` crate (v0.25.10) for thumbnail generation. The existing naive-ui component library already provides autocomplete functionality, and zoom/pan can be implemented with CSS transforms without additional packages.

**The existing codebase provides strong foundations:** Download pause/resume is already fully implemented. Image preview modals follow established patterns in Home.vue and Gallery.vue. The architecture is sound; risks center on memory management (base64 encoding) and unbounded data structures rather than fundamental design issues.

**Recommended phase order:** Viewer enhancements first (user-facing, moderate complexity), then tag autocomplete (improves search UX), then download retry button (trivial - just UI), then gallery indexing with thumbnails last (most complex, background task).

---

## Key Findings

### Stack Additions

| Component | Technology | Version | Rationale |
|-----------|------------|---------|-----------|
| Thumbnail generation | `image` crate (Rust) | 0.25.10 | Pure Rust, async via spawn_blocking |
| Zoom/pan (MVP) | CSS transform | native | Hardware accelerated, zero deps |
| Tag autocomplete | NAutoComplete | built-in | Part of naive-ui 2.44.1 |
| Zoom library (optional) | vue-panzoom | 1.1.6 | Only if CSS zoom insufficient |

**Only one new dependency required:** `cargo add image` in src-tauri.

### Feature Classification

| Feature | Classification | Rationale |
|---------|----------------|-----------|
| Zoom/Pan controls | **Table stakes** | Users expect to inspect images in detail |
| Keyboard navigation | **Table stakes** | Already exists, just enhance |
| Fit/Fill/Actual modes | **Table stakes** | Standard viewer behavior |
| Tag autocomplete | **Table stakes** | Improves search UX significantly |
| Download retry button | **Table stakes** | Backend already supports, needs UI |
| Filmstrip/thumbnail strip | **Differentiator** | Defer to Phase 2 |
| Fullscreen mode | **Differentiator** | Defer to Phase 2 |
| Fuzzy tag matching | **Differentiator** | Nice-to-have, use fuse.js if added |
| Auto-retry toggle | **Differentiator** | Settings feature, post-MVP |
| Background thumbnail generation | **Differentiator** | Complex, defer to Phase 3 |

### Architecture Integration Points

**Image Viewer:**
- New: `ImageViewer.vue`, `useImageViewer.ts`
- Modify: `Home.vue`, `Gallery.vue` (replace inline preview with ImageViewer)
- Pattern: Reuse modal pattern (v-if overlay, not new route)
- Data flow: `convertFileSrc()` for local, `get_image_base64` for remote

**Tag Autocomplete:**
- New: `useTagAutocomplete.ts`, `commands/tags.rs`
- Modify: `Home.vue`, `db/mod.rs` (add tag_hints table)
- Cache: LRU in frontend + SQLite for persistence
- Flow: User history > LRU cache > backend fetch > live search

**Download Pause/Resume:**
- **NO CHANGES NEEDED** - Fully implemented
- State persistence: SQLite via `save_download_task()`
- Channels: `pause_tokens` + `pause_rx` already wired

**Gallery Indexing:**
- New: `commands/indexer.rs`, `useGalleryIndex.ts`
- Modify: `Gallery.vue`, `db/mod.rs` (add gallery_index table)
- Approach: Hybrid (background indexer + on-demand refresh)
- Events: `index-progress`, `index-complete`

### Critical Pitfalls and Prevention

| Pitfall | Severity | Prevention |
|---------|----------|------------|
| Base64 memory explosion | CRITICAL | Use `convertFileSrc()` asset protocol instead of `get_image_base64` for local images |
| Unbounded tag list | CRITICAL | Virtual scrolling + debounced search (3+ chars) + LRU cache limits |
| Download state desync | CRITICAL | Add `resume_offset` column to downloads table; persist byte offset on pause |
| Thumbnail blocks main thread | CRITICAL | Background worker in Rust with `spawn_blocking`; emit progress events |
| Race condition in progress events | MODERATE | Use `Map<id, task>` instead of array; debounce UI updates |
| Missing error boundary | MODERATE | Add `@error` handler + fallback placeholder + retry button |
| Path collision | MODERATE | Add timestamp suffix to filename; check for existing file |
| Scroll lock missing | MINOR | Add `overflow: hidden` to preview overlay |
| Keyboard nav conflicts | MINOR | Prevent default on arrow keys when autocomplete open |

---

## Implications for Roadmap

### Phase 1: Image Viewer Enhancement (2-3 weeks)

**Rationale:** High user impact, moderate complexity, established patterns to follow.

**Deliverables:**
- `ImageViewer.vue` component with zoom/pan
- `useImageViewer.ts` composable
- CSS transform zoom (no new dependencies)
- Keyboard shortcuts: +/-, 0, f, arrows
- Fit/Fill/Actual size modes

**Must avoid:**
- Base64 memory explosion (use asset protocol)
- Scroll lock on modal open

### Phase 2: Tag Autocomplete (1-2 weeks)

**Rationale:** Moderate complexity, improves core search UX, existing patterns.

**Deliverables:**
- `useTagAutocomplete.ts` composable
- `commands/tags.rs` backend with SQLite cache
- NAutoComplete integration in Home.vue
- Debounced search (300ms), fuzzy matching optional

**Must avoid:**
- Unbounded tag list (virtual scrolling if >100 items)
- Keyboard nav conflicts with browser

### Phase 3: Download Retry UI (1 day)

**Rationale:** Trivial - backend already supports.

**Deliverables:**
- Retry button on failed tasks in UI
- Error message display
- Auto-retry toggle in settings (optional)

**Must avoid:**
- None significant

### Phase 4: Gallery Indexing (3-4 weeks)

**Rationale:** Most complex, benefits large libraries, can be incremental.

**Deliverables:**
- `commands/indexer.rs` with background indexing
- `gallery_index` SQLite table
- `useGalleryIndex.ts` composable
- Progress events for UI
- Thumbnail generation with `image` crate

**Must avoid:**
- Blocking main thread (use async Rust)
- Unbounded memory (semaphore limits concurrent ops)

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Single Rust crate, proven versions |
| Features | MEDIUM-HIGH | Based on codebase analysis + industry patterns |
| Architecture | HIGH | Existing patterns strong, download fully implemented |
| Pitfalls | HIGH | Based on debug history + code review |

**Gaps to address during planning:**
1. Verify Range header support for partial download resume
2. Confirm thumbnail cache directory structure
3. Test base64 memory behavior with 10MB+ images

---

## Research Flags

| Phase | Deep Research Needed | Standard Patterns |
|-------|---------------------|-------------------|
| Phase 1 (Viewer) | No | Yes - CSS zoom/pan well documented |
| Phase 2 (Autocomplete) | No | Yes - naive-ui autocomplete pattern |
| Phase 3 (Retry UI) | No | Yes - just button + handler |
| Phase 4 (Thumbnails) | Maybe | Partial - image crate API needs verification |

---

## Open Questions

1. **Download resume:** Does Gelbooru support Range headers for partial resume? If not, `resume_offset` is irrelevant.
2. **Thumbnail sizes:** What dimensions to generate? (150x150 for grid, 300x300 for filmstrip?)
3. **Fullscreen approach:** Tauri window mode vs HTML5 Fullscreen API? Tauri provides better control but requires window management.
4. **Tag hint source:** Use Gelbooru `/index.php?page=tag&s=案&q={query}` or local SQLite? Hybrid approach recommended.

---

## Sources

- STACK.md: Technology recommendations based on existing codebase + npm/Cargo registries
- FEATURES.md: Feature analysis based on codebase gaps + industry standards
- ARCHITECTURE.md: Integration patterns from Home.vue, Gallery.vue, download.rs, gallery.rs
- PITFALLS.md: Debug history + code review of critical code paths