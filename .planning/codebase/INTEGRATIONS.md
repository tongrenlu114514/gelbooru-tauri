# External Integrations

**Analysis Date:** 2026-04-13

## APIs & External Services

**Gelbooru Website:**
- Gelbooru API (web scraping)
  - Base URL: `https://gelbooru.com`
  - Implementation: `src-tauri/src/services/scraper.rs`
  - Commands: `src-tauri/src/commands/gelbooru.rs`
  - Endpoints used:
    - Search: `/index.php?page=post&s=list&tags={}&pid={}`
    - Post detail: `/index.php?page=post&s=view&id={}`

**Image Proxies:**
- Default proxy: `http://127.0.0.1:7897`
  - Configurable via Tauri command `set_proxy`
  - Implementation: `src-tauri/src/services/http.rs`

**Image CDNs (Gelbooru):**
- `img2.gelbooru.com` - Image hosting
- Support for: PNG, GIF, WebP, JPEG
- Referer header required for downloads

## Data Storage

**SQLite Database:**
- Location: `{app_dir}/gelbooru.db`
- Implementation: `src-tauri/src/db/mod.rs`
- Client: rusqlite 0.32 (bundled)

**Tables:**
```sql
downloads (
  id INTEGER PRIMARY KEY,
  post_id INTEGER,
  file_name TEXT,
  file_path TEXT,
  image_url TEXT,
  status TEXT,
  progress REAL,
  downloaded_size INTEGER,
  total_size INTEGER,
  created_at TIMESTAMP,
  completed_at TIMESTAMP,
  error_message TEXT
)

favorites (
  id INTEGER PRIMARY KEY,
  post_id INTEGER UNIQUE,
  created_at TIMESTAMP
)

favorite_tags (
  id INTEGER PRIMARY KEY,
  tag TEXT UNIQUE,
  tag_type TEXT,
  parent_id INTEGER,
  created_at TIMESTAMP
)

blacklisted_tags (
  id INTEGER PRIMARY KEY,
  tag TEXT UNIQUE,
  created_at TIMESTAMP
)
```

**File Storage:**
- Downloaded images stored in user-configured paths
- Local gallery browses filesystem
- Image caching in frontend memory (Map-based)

## Authentication & Identity

**Cookie-Based Auth:**
- Implementation: `src-tauri/src/services/http.rs`
- Cookie jar: `reqwest::cookie::Jar`
- Method: `load_cookies()` - loads cookies from JSON file
- Supports browser cookie import for accessing NSFW content

**Session Management:**
- Persistent cookies across requests
- Shared cookie jar for HTTP client

## Browser Integration

**Asset Protocol:**
- Enabled for local file serving
- Scope: All files allowed
- Implementation: `convertFileSrc()` from Tauri API
- Used in: `src/views/Gallery.vue`

## Native Desktop Capabilities

**File System Access:**
- Plugin: `@tauri-apps/plugin-fs` (v2.4.5)
- Commands in `src-tauri/src/commands/gallery.rs`:
  - `get_local_images` - List local images
  - `get_directory_tree` - Browse directory structure
  - `get_directory_images` - Get images in directory
  - `delete_image` - Delete local files
  - `get_local_image_base64` - Read image as base64
  - `open_file` - Open file with default application

**Shell Integration:**
- Plugin: `@tauri-apps/plugin-shell` (v2.3.5)
- Platform-specific file opening:
  - Windows: `cmd /C start`
  - macOS: `open`
  - Linux: `xdg-open`

## Tauri Event System

**Frontend Events (Emitted):**
- `download-task-added` - New download task created
- `download-progress` - Download progress updates

**Backend Commands (Invoked):**
- Gelbooru: `search_posts`, `get_post_detail`, `get_image_base64`, `set_proxy`
- Downloads: `add_download_task`, `start_download`, `pause_download`, `resume_download`, `cancel_download`, `remove_download_task`, `get_download_tasks`, `clear_completed_tasks`, `open_file`
- Gallery: `get_local_images`, `delete_image`, `get_directory_tree`, `get_directory_images`, `get_local_image_base64`
- Favorite Tags: `get_favorite_tags`, `add_parent_tag`, `add_child_tag`, `remove_favorite_tag`, `is_tag_favorited`, `get_child_tags`

## Download Manager

**Architecture:** `src-tauri/src/commands/download.rs`
- Global manager: `DOWNLOAD_MANAGER` (lazy_static)
- Concurrency: Semaphore-based (default 3 concurrent downloads)
- Progress tracking: Event-based via Tauri
- Cancellation: mpsc channel pattern

**Download Flow:**
1. `add_download_task` - Create task
2. `start_download` - Begin download with semaphore
3. Progress events emitted every ~100KB
4. Temp files used during download (`.tmp` extension)
5. Atomic rename on completion

## Environment Configuration

**Required at Runtime:**
- Database directory (auto-created)
- Download path (user-configured)
- Optional proxy URL

**Dev Configuration:**
- Dev server: `http://localhost:1420`
- Build command: `pnpm build`

---

*Integration audit: 2026-04-13*
