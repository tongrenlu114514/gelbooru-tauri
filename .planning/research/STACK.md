# Technology Stack

**Project:** Gelbooru Downloader (v1.2 — Viewer & Indexing)
**Researched:** 2026-05-12

---

## Recommended Stack

### Core Framework
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Vue 3 | 3.5.x | UI framework | Already in use |
| Pinia | 2.3.x | State management | Already in use |
| naive-ui | 2.44.1 | Component library | Already in use |
| Tauri 2.x | 2.x | Desktop framework | Already in use |

### Image Viewer
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| CSS transform | native | Zoom/pan MVP | Zero dependency, hardware accelerated |
| vue-panzoom | 1.1.6 | Full zoom features | Only if CSS zoom insufficient (optional) |

### Tag Autocomplete
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| NAutoComplete | built-in | Tag suggestions | Part of naive-ui 2.44.1, no new package |

### Thumbnail Generation
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| image (Rust) | 0.25.10 | Thumbnail generation | Pure Rust, async via spawn_blocking |

### Database
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| rusqlite | 0.32 | Gallery cache | Already in use, add gallery_cache table |

---

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Zoom library | CSS-only MVP | vue-panzoom | Overkill for basic zoom; CSS transform handles 80% of use cases |
| Zoom library | vue-panzoom (if needed) | @zoom-image/vue | vue-panzoom has more Vue 3 compatible API |
| Autocomplete | NAutoComplete | vue3-autocomplete | naive-ui built-in is sufficient |
| Image processing | image crate | imageflow | image crate is simpler, Rust-native |

---

## Installation

### Frontend (optional)
```bash
# Only if CSS zoom is insufficient
pnpm add vue-panzoom
```

### Backend (Rust)
```bash
cd src-tauri && cargo add image
```

---

## Source

- vue-panzoom: npm v1.1.6 (verified 2026-05-12)
- naive-ui: v2.44.1 installed (verified via pnpm list)
- image crate: v0.25.10 (verified via cargo info)