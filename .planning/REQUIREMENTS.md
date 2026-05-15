# Requirements: Gelbooru Downloader

**Defined:** 2026-05-12
**Core Value:** 图片搜索和下载，带本地图库浏览

## v1.2 Requirements

### Image Viewer (UI)

- [ ] **UI-01**: User can view image in fullscreen modal overlay
- [ ] **UI-02**: User can navigate between images using left/right buttons or keyboard
- [ ] **UI-03**: User can zoom in/out using mouse wheel or pinch gesture
- [ ] **UI-04**: User can pan/drag zoomed image to view details
- [ ] **UI-05**: User can use keyboard shortcuts (ArrowLeft, ArrowRight, Escape, +/-)
- [ ] **UI-06**: User can view filmstrip showing neighboring images for quick navigation

### Tag Autocomplete (TAG)

- [ ] **TAG-01**: User sees tag autocomplete suggestions while typing
- [ ] **TAG-02**: User receives tag recommendations based on search history

### Download Management (DL)

- [ ] **DL-01**: User can pause and resume download tasks
- [ ] **DL-02**: User can retry failed download tasks with one click

### Gallery Indexing (IDX)

- [ ] **IDX-01**: App maintains SQLite index of local gallery images
- [ ] **IDX-02**: App stores thumbnails in dedicated cache directory
- [ ] **IDX-03**: App generates thumbnails on-demand when viewing gallery
- [ ] **IDX-04**: App generates thumbnails in background for faster subsequent loading

## v2 Requirements

*(Moved to v1.2 scope - see above)*

## Out of Scope

| Feature | Reason |
|---------|--------|
| Virtual scrolling for tag list | Defer to future enhancement |
| Partial file resume (Range header) | Backend supports pause, but server may not support Range |
| Cloud sync | Storage costs, not core value |
| Video support | Different handling required, defer |
| RAW format support | Specialized, not priority |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| UI-01 | Phase 7 | Pending |
| UI-02 | Phase 7 | Pending |
| UI-03 | Phase 7 | Pending |
| UI-04 | Phase 7 | Pending |
| UI-05 | Phase 7 | Pending |
| UI-06 | Phase 7 | Pending |
| TAG-01 | Phase 8 | Pending |
| TAG-02 | Phase 8 | Pending |
| DL-01 | Phase 9 | Pending |
| DL-02 | Phase 9 | Pending |
| IDX-01 | Phase 10 | Pending |
| IDX-02 | Phase 11 | Pending |
| IDX-03 | Phase 10 | Pending |
| IDX-04 | Phase 11 | Pending |

**Coverage:**
- v1.2 requirements: 14 total (6 UI + 2 TAG + 2 DL + 4 IDX)
- Mapped to phases: 14/14
- Unmapped: 0

---
*Requirements defined: 2026-05-12*
*Last updated: 2026-05-13 after roadmap created*