# Gelbooru Downloader - Roadmap

**Last Updated:** 2026-05-13 after v1.2 roadmap created

## Milestones

- ✅ **v1.0 MVP** — Phases 1-4 (shipped 2026-05-10)
- ✅ **v1.1 UI** — Phases 5-6 (shipped 2026-05-12)
- 🚧 **v1.2 Viewer & Indexing** — Phases 7-10 (in progress)
- 📋 **v2.0** — Future (planned)

## Phases

### 🚧 v1.2 Viewer & Indexing (Phases 7-10)

- [ ] **Phase 7: Image Viewer Enhancement** — Fullscreen viewer with zoom/pan (UI-01 to UI-06)
- [ ] **Phase 8: Tag Autocomplete** — Search tag autocomplete and recommendations (TAG-01, TAG-02)
- [x] **Phase 9: Download Retry UI** — Retry button and pause/resume controls (DL-01, DL-02) (completed 2026-05-14)
- [x] **Phase 10: Gallery Indexing** — SQLite index and thumbnail generation (IDX-01 to IDX-04) (completed 2026-05-14)

## Phase Details

### Phase 7: Image Viewer Enhancement
**Goal**: Users can view images in fullscreen with zoom/pan controls and keyboard navigation
**Depends on**: Nothing
**Requirements**: UI-01, UI-02, UI-03, UI-04, UI-05, UI-06
**Success Criteria** (what must be TRUE):
1. User can open image in fullscreen modal overlay by clicking thumbnail
2. User can zoom in/out using mouse wheel or keyboard (+/-)
3. User can pan/drag zoomed image to view details
4. User can navigate between images using left/right buttons or keyboard arrows
5. User can close viewer with Escape key
6. User can view filmstrip showing neighboring images for quick navigation
**Plans**: 2 plans
Plans:
- [x] 07-01-PLAN.md — ImageViewer.vue with zoom/pan/keyboard
- [x] 07-02-PLAN.md — Filmstrip.vue component
**UI hint**: yes

### Phase 8: Tag Autocomplete
**Goal**: Users can search with tag autocomplete suggestions and recommendations
**Depends on**: Phase 7
**Requirements**: TAG-01, TAG-02
**Success Criteria** (what must be TRUE):
1. User sees tag autocomplete dropdown while typing in search tag input
2. User can select suggestion with Enter key or mouse click
3. User receives tag recommendations based on search history
**Plans**: 1 plan
Plans:
- [ ] 08-01-PLAN.md — TagAutocompleteInput + SearchHistoryStore
**UI hint**: yes

### Phase 9: Download Retry UI
**Goal**: Users can retry failed downloads and manage download tasks
**Depends on**: Phase 8
**Requirements**: DL-01, DL-02
**Success Criteria** (what must be TRUE):
1. User can pause active download tasks and resume paused tasks
2. User can retry failed download tasks with one click
3. User sees error message displayed for failed downloads
**Plans**: 1 plan
Plans:
- [x] 09-01-PLAN.md — Download Retry UI with retry action, error display, pause/resume, failed filter
**UI hint**: yes

### Phase 10: Gallery Indexing
**Goal**: App maintains SQLite index of local images and generates thumbnails for fast gallery loading
**Depends on**: Phase 9
**Requirements**: IDX-01, IDX-02, IDX-03, IDX-04
**Success Criteria** (what must be TRUE):
1. App maintains SQLite index of all local gallery images
2. App stores generated thumbnails in dedicated cache directory
3. App generates thumbnails on-demand when viewing gallery for uncached images
4. App generates thumbnails in background for faster subsequent loading
**Plans**: 1 plan
Plans:
- [x] 10-01-PLAN.md — Backend: SQLite schema + thumbnail generation + Tauri commands

## Progress

| Phase | Milestone | Plans Complete | Status |
|-------|-----------|---------------|--------|
| 1. Foundation | v1.0 | 1/1 | Complete |
| 2. Quality & Testing | v1.0 | 5/5 | Complete |
| 3. Performance & Reliability | v1.0 | 4/4 | Complete |
| 4. Polish & Release | v1.0 | 2/2 | Complete |
| 5. Gallery Redesign | v1.1 | 1/1 | Complete |
| 6. Masonry + Breadcrumb | v1.1 | 2/2 | Complete |
| 7. Image Viewer Enhancement | v1.2 | 2/2 | Complete |
| 8. Tag Autocomplete | v1.2 | 0/1 | Not started |
| 9. Download Retry UI | 1/1 | Complete    | 2026-05-14 |
| 10. Gallery Indexing | 1/1 | Complete    | 2026-05-14 |