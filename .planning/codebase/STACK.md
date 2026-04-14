# Technology Stack

**Analysis Date:** 2026-04-13

## Languages

**Primary:**
- TypeScript 5.7.3 - Frontend (Vue 3)
- Rust 2021 edition - Backend (Tauri)

## Runtime

**Frontend Environment:**
- Node.js (via Vite dev server)
- pnpm - Package manager

**Desktop Runtime:**
- Tauri runtime - Native desktop application container

## Frontend Stack

**Core:**
- Vue 3.5.13 - UI framework with Composition API
- TypeScript 5.7.3 - Type safety
- Pinia 2.3.0 - State management (store pattern)
- Vue Router 4.5.0 - Client-side routing

**UI Library:**
- naive-ui 2.41.0 - Component library
  - Uses `NLayout`, `NButton`, `NIcon`, `NSpace`, `NTree`, `NModal`, `NSpin`, `NEmpty`, `NText`, `NLayoutSider`, `NLayoutContent`
- @vicons/ionicons5 0.12.0 - Icon set

**Build Tool:**
- Vite 6.0.11 - Build tool and dev server
- @vitejs/plugin-vue 5.2.1 - Vue SFC plugin
- vue-tsc 2.2.0 - TypeScript checking

**Tauri Integration:**
- @tauri-apps/api 2.2.0 - Tauri frontend API
- @tauri-apps/plugin-fs 2.4.5 - File system access
- @tauri-apps/plugin-shell 2.3.5 - Shell commands

## Backend Stack

**Framework:**
- Tauri 2.x - Desktop app framework
  - Uses `tauri-plugin-fs` for file operations
  - Uses `tauri-plugin-shell` for shell access
  - Asset protocol enabled for local file serving

**HTTP Client:**
- reqwest 0.12 - Async HTTP client
  - Features: cookies, gzip, json, stream, rustls-tls-webpki-roots
  - Cookie jar for session management
  - Support for Referer headers (image downloads)

**Database:**
- rusqlite 0.32 - SQLite bindings with bundled SQLite
  - Used for: downloads, favorites, favorite_tags tables

**HTML Parsing:**
- scraper 0.22 - HTML parsing and CSS selector queries

**Async Runtime:**
- tokio 1.x - Async runtime with full features
  - Used: RwLock, mpsc, Semaphore

**Serialization:**
- serde 1.x - Serialization/deserialization
- serde_json 1.x - JSON support

**Utilities:**
- regex 1.x - Regular expressions
- url 2.x - URL parsing
- chrono 0.4 - Date/time handling
- base64 0.22 - Base64 encoding
- lazy_static 1.5 - Static initialization
- thiserror 1.x - Error handling
- async-trait 0.1 - Async trait support
- futures 0.3 - Async utilities

## Build Configuration

**Tauri Config:** `src-tauri/tauri.conf.json`
- App identifier: `com.gelbooru.downloader`
- Window: 1200x800, min 800x600
- CSP: Allows img-src *, connect-src *, inline scripts/styles
- NSIS installer for Windows

**Cargo Profile:**
- Release optimizations: LTO, opt-level "s", stripped binaries
- Panic: abort (smaller binaries)

## Project Structure

```
gelbooru/
├── src/                    # Frontend (Vue 3)
│   ├── main.ts             # App entry point
│   ├── App.vue             # Root component
│   ├── router/index.ts     # Vue Router setup
│   ├── stores/             # Pinia stores
│   │   ├── gallery.ts      # Gallery state
│   │   ├── download.ts     # Download state
│   │   ├── settings.ts     # Settings state
│   │   └── favoriteTags.ts # Favorite tags state
│   ├── views/              # Page components
│   │   ├── Home.vue        # Search page
│   │   ├── Gallery.vue     # Local gallery
│   │   ├── Downloads.vue   # Download manager
│   │   ├── FavoriteTags.vue # Tag favorites
│   │   └── Settings.vue    # App settings
│   ├── components/         # Reusable components
│   │   └── DownloadNotifier.vue
│   └── types/index.ts      # TypeScript interfaces
├── src-tauri/
│   ├── src/
│   │   ├── main.rs        # Tauri entry point
│   │   ├── lib.rs         # Library root
│   │   ├── commands/       # Tauri commands
│   │   │   ├── gelbooru.rs    # Gelbooru API
│   │   │   ├── download.rs     # Download manager
│   │   │   ├── gallery.rs      # Local gallery
│   │   │   └── favorite_tags.rs # Tag favorites
│   │   ├── services/       # Business logic
│   │   │   ├── http.rs     # HTTP client
│   │   │   └── scraper.rs  # HTML scraper
│   │   ├── models/         # Data models
│   │   │   ├── post.rs
│   │   │   ├── tag.rs
│   │   │   └── page.rs
│   │   └── db/             # Database
│   │       └── mod.rs      # SQLite operations
│   ├── Cargo.toml
│   └── tauri.conf.json
└── package.json
```

---

*Stack analysis: 2026-04-13*
