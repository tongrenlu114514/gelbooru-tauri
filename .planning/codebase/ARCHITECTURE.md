# Architecture

**Analysis Date:** 2026-04-13

## Pattern Overview

**Overall:** Tauri Desktop Application with Vue.js Frontend + Rust Backend

**Key Characteristics:**
- Multi-process architecture: WebView (renderer) + Rust (native)
- IPC communication via Tauri commands and events
- Reactive frontend with Pinia state management
- HTML scraping for Gelbooru API (unofficial integration)
- SQLite local database for persistence

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        Desktop Window                            │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                    Vue.js Frontend                        │  │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────┐  │  │
│  │  │ Router  │  │ Pinia   │  │ NaiveUI │  │  Views/     │  │  │
│  │  │         │  │ Stores  │  │         │  │  Components │  │  │
│  │  └────┬────┘  └────┬────┘  └─────────┘  └─────────────┘  │  │
│  │       │             │                                      │  │
│  │       └─────────────┼──────────────────────────────────    │  │
│  │                     │                                      │  │
│  │              @tauri-apps/api                               │  │
│  │                     │                                      │  │
│  └─────────────────────┼──────────────────────────────────────┘  │
│                        │                                        │
│                   IPC Bridge                                     │
│              (invoke / emit/listen)                             │
│                        │                                        │
│  ┌─────────────────────┼──────────────────────────────────────┐  │
│  │                     │           Rust Backend               │  │
│  │  ┌──────────────────┴───────────────────────────────────┐  │  │
│  │  │              Tauri Command Handler                    │  │  │
│  │  └──────────────────┬───────────────────────────────────┘  │  │
│  │                     │                                      │  │
│  │  ┌──────────────────┴───────────────────────────────────┐  │  │
│  │  │                    Commands                          │  │  │
│  │  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐  │  │  │
│  │  │  │  gelbooru   │ │   download   │ │   gallery    │  │  │  │
│  │  │  │  commands   │ │   commands   │ │   commands   │  │  │  │
│  │  │  └─────────────┘ └─────────────┘ └─────────────┘  │  │  │
│  │  │  ┌─────────────┐ ┌─────────────┐                  │  │  │
│  │  │  │favorite_tags│ │             │                  │  │  │
│  │  │  │  commands   │ │             │                  │  │  │
│  │  │  └──────┬──────┘ └─────────────┘                  │  │  │
│  │  └─────────┼───────────────────────────────────────────┘  │  │
│  │            │                                               │  │
│  │  ┌─────────┴───────────────────────────────────────────┐  │  │
│  │  │                    Services                          │  │  │
│  │  │  ┌─────────────┐ ┌─────────────┐                  │  │  │
│  │  │  │   Scraper   │ │    HTTP     │                  │  │  │
│  │  │  │   Service   │ │   Client    │                  │  │  │
│  │  │  └─────────────┘ └─────────────┘                  │  │  │
│  │  └─────────────────────────────────────────────────────┘  │  │
│  │                                                           │  │
│  │  ┌─────────────────────────────────────────────────────┐  │  │
│  │  │                    Database                          │  │  │
│  │  │                  (SQLite)                            │  │  │
│  │  └─────────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## Layers

### Frontend Layer (Vue.js)

**Purpose:** User interface and interaction handling

**Location:** `src/`

**Contains:**
- Views: Page components (Home, Gallery, Downloads, Settings, FavoriteTags)
- Components: Reusable UI components (AppSidebar, DownloadNotifier)
- Stores: Pinia state management
- Router: Vue Router configuration
- Types: TypeScript interfaces

**Depends on:** NaiveUI components, Pinia, Vue Router

**Used by:** Tauri WebView

### Command Layer (Rust)

**Purpose:** Bridge between frontend and backend services

**Location:** `src-tauri/src/commands/`

**Contains:**
- `gelbooru.rs`: Gelbooru search and scraping commands
- `download.rs`: Download task management commands
- `gallery.rs`: Local file system browsing commands
- `favorite_tags.rs`: Favorite tag CRUD commands

**Depends on:** Services, Models, Database

**Used by:** Frontend via `invoke()`

### Service Layer (Rust)

**Purpose:** Business logic and external integrations

**Location:** `src-tauri/src/services/`

**Contains:**
- `scraper.rs`: HTML parsing for Gelbooru pages
- `http.rs`: HTTP client with proxy support and cookie management

**Depends on:** Models, reqwest

**Used by:** Commands

### Model Layer (Rust)

**Purpose:** Data structures shared across the application

**Location:** `src-tauri/src/models/`

**Contains:**
- `post.rs`: GelbooruPost, GelbooruPostStatistics
- `tag.rs`: GelbooruTag
- `page.rs`: GelbooruPage

**Used by:** Commands, Services, Frontend

### Database Layer (Rust)

**Purpose:** Persistent storage with SQLite

**Location:** `src-tauri/src/db/mod.rs`

**Contains:**
- Database connection management
- Tables: downloads, favorites, blacklisted_tags, favorite_tags
- CRUD operations for favorite tags

**State Management:** Thread-safe via `Mutex<Connection>`

## Frontend-Backend Communication

### Command Invocation (Request-Response)

Frontend calls Rust commands using `invoke()`:

```typescript
// src/stores/gallery.ts
const result = await invoke<{ postList: GelbooruPost[], tagList: GelbooruTag[], totalPages: number }>('search_posts', {
  tags: tags,
  page: galleryStore.currentPage
})
```

Backend handler in `src-tauri/src/commands/gelbooru.rs`:

```rust
#[tauri::command]
pub async fn search_posts(tags: Vec<String>, page: u32) -> Result<SearchResult, String> {
    let client = HTTP_CLIENT.read().await;
    let url = SCRAPER.build_search_url(&tags, page);
    let html = client.get(&url).await.map_err(|e| format!("HTTP request failed: {}", e))?;
    let (post_list, tag_list, total_pages) = SCRAPER.parse_page(&html);
    Ok(SearchResult { post_list, tag_list, total_pages })
}
```

### Event Emission (Push Updates)

Backend pushes download progress to frontend using `emit()`:

```rust
// src-tauri/src/commands/download.rs
fn emit_progress(app: &AppHandle, id: u32, post_id: u32, status: &str, ...) {
    let event = DownloadProgressEvent { ... };
    let _ = app.emit("download-progress", &event);
}
```

Frontend listens via `listen()`:

```typescript
// src/stores/download.ts
unlistenProgress = await listen<DownloadProgressEvent>('download-progress', (event) => {
  const data = event.payload
  // Update task in store
})
```

### Registered Commands

From `src-tauri/src/main.rs`:

```rust
invoke_handler(tauri::generate_handler![
    // Gelbooru
    commands::gelbooru::search_posts,
    commands::gelbooru::get_post_detail,
    commands::gelbooru::get_image_base64,
    commands::gelbooru::set_proxy,
    // Download
    commands::download::add_download_task,
    commands::download::start_download,
    commands::download::pause_download,
    commands::download::resume_download,
    commands::download::cancel_download,
    commands::download::remove_download_task,
    commands::download::get_download_tasks,
    commands::download::clear_completed_tasks,
    commands::download::open_file,
    // Gallery
    commands::gallery::get_local_images,
    commands::gallery::delete_image,
    commands::gallery::get_directory_tree,
    commands::gallery::get_directory_images,
    commands::gallery::get_local_image_base64,
    // Favorite Tags
    commands::favorite_tags::get_favorite_tags,
    commands::favorite_tags::add_parent_tag,
    commands::favorite_tags::add_child_tag,
    commands::favorite_tags::remove_favorite_tag,
    commands::favorite_tags::is_tag_favorited,
    commands::favorite_tags::get_child_tags,
])
```

## State Management (Pinia)

### Store Architecture

```
src/stores/
├── gallery.ts      # Posts, tags, pagination state
├── download.ts     # Download tasks, progress tracking
├── settings.ts     # App settings (theme, paths, proxy)
└── favoriteTags.ts # Favorite tag groups
```

### Store Patterns

**Setup Stores (Composition API):**

```typescript
// src/stores/settings.ts
export const useSettingsStore = defineStore('settings', () => {
  const theme = ref<'light' | 'dark'>('dark')
  const sidebarCollapsed = ref(false)
  // ...
  return { theme, sidebarCollapsed, toggleTheme, ... }
})
```

**Reactive Stores with Backend Events:**

```typescript
// src/stores/download.ts
export const useDownloadStore = defineStore('download', () => {
  const tasks = ref<DownloadTask[]>([])
  let unlistenProgress: UnlistenFn | null = null

  async function initListeners() {
    unlistenProgress = await listen<DownloadProgressEvent>('download-progress', (event) => {
      // Update task from event
    })
  }

  return { tasks, initListeners, addTask, ... }
})
```

### Page State Recovery

Gallery store maintains page state for navigation:

```typescript
// src/stores/gallery.ts
interface PageState {
  selectedTags: string[]
  selectedRating: string
  currentPage: number
  posts: GelbooruPost[]
  // ...
}

function savePageState(selectedTags: string[], selectedRating: string) {
  pageState.value = { selectedTags, selectedRating, currentPage: currentPage.value, ... }
}

function restorePageState(): PageState | null {
  return pageState.value
}
```

Used in `Home.vue` with route guards:

```typescript
// src/views/Home.vue
onBeforeRouteLeave(() => {
  galleryStore.savePageState(selectedTags.value, selectedRating.value)
})
```

## Data Flow

### Search Flow

```
User Input (tags)
    │
    ▼
Home.vue: searchPosts()
    │
    │ invoke('search_posts', { tags, page })
    ▼
gelbooru.rs: search_posts()
    │
    ├── HTTP_CLIENT.get(url)
    │       │
    │       ▼
    │   Gelbooru API (HTML response)
    │       │
    │       ▼
    │   scraper.rs: parse_page(html)
    │       │
    ▼       ▼
    │
    ▼
SearchResult { postList, tagList, totalPages }
    │
    ▼
Home.vue: galleryStore.setPosts(result.postList)
    │
    ▼
UI Update (post-grid rendering)
```

### Download Flow

```
User clicks "Download"
    │
    ▼
Home.vue: downloadPost(post)
    │
    │ invoke('add_download_task', { postId, imageUrl, ... })
    ▼
download.rs: add_download_task()
    │
    ├── Creates DownloadTask
    ├── DOWNLOAD_MANAGER.add_task(task)
    ├── app.emit('download-task-added', task)
    ▼
Returns DownloadTask to frontend
    │
    ▼
Frontend adds to store, auto-starts
    │
    │ invoke('start_download', { id })
    ▼
download.rs: start_download()
    │
    ├── Spawns async task
    ├── Acquires semaphore (concurrency limit)
    ├── HTTP_CLIENT.download_image()
    ├── Streams to temp file
    ├── emit('download-progress', progress)
    │       │
    │       ▼
    │   Frontend updates store
    │
    ├── On complete: rename temp -> final
    ├── emit('download-progress', { status: 'completed' })
    ▼
Frontend shows notification
```

### Favorite Tags Flow

```
User adds favorite tag
    │
    │ invoke('add_parent_tag', { tag, tagType })
    ▼
favorite_tags.rs: add_parent_tag()
    │
    ├── State<DbState> (thread-safe database access)
    ▼
Database.add_favorite_tag()
    │
    └── INSERT INTO favorite_tags (tag, tag_type)
    ▼
Returns new tag ID
    │
    ▼
Frontend reloads tags: invoke('get_favorite_tags')
```

## Key Design Decisions

### 1. HTML Scraping vs API

**Decision:** Use HTML scraping instead of official API

**Rationale:**
- Official API requires authentication and has rate limits
- HTML scraping works with cookies from browser
- Allows fetching without authentication

**Implementation:** `scraper.rs` parses HTML with `scraper` crate

### 2. Download Manager Pattern

**Decision:** In-memory download manager with SQLite backup

**Rationale:**
- Downloads need to persist across frontend reloads
- In-memory allows fast state access
- SQLite provides durability

**Implementation:** `DownloadManager` singleton with `RwLock<HashMap<u32, DownloadTask>>`

### 3. Proxy Support

**Decision:** Global HTTP client with dynamic proxy configuration

**Rationale:**
- Gelbooru may be blocked in some regions
- User can configure proxy in settings
- Proxy applies to all HTTP requests

**Implementation:** `HttpClient` wraps `reqwest::Client` with proxy middleware

### 4. Favorite Tags Hierarchy

**Decision:** Parent-child relationship for tags (e.g., copyright -> character)

**Rationale:**
- Users want to organize tags by series/character
- Quick selector needs hierarchical data
- Enables cascade delete

**Implementation:** `favorite_tags` table with `parent_id` foreign key

### 5. Base64 Image Encoding

**Decision:** Encode remote/local images as base64 data URLs

**Rationale:**
- Vue `<img>` src requires data URL or same-origin
- Avoids CORS issues with remote images
- Simplifies local image display

**Implementation:**
- `get_image_base64()`: Fetch remote image via backend
- `get_local_image_base64()`: Read local file, encode

## Error Handling

### Frontend

```typescript
// Pattern: try-catch with user feedback
try {
  await invoke('start_download', { id })
} catch (error) {
  console.error('Failed to start download:', error)
  message.error('启动下载失败')
}
```

### Backend

```rust
// Pattern: Result<T, String> with descriptive errors
#[tauri::command]
pub async fn search_posts(tags: Vec<String>, page: u32) -> Result<SearchResult, String> {
    let html = client.get(&url).await
        .map_err(|e| format!("HTTP request failed: {}", e))?;
    // ...
    Ok(SearchResult { ... })
}
```

## Cross-Cutting Concerns

### Logging

Backend uses `println!()` for debugging:

```rust
println!("[DEBUG] Fetching URL: {}", url);
println!("[ERROR] {}", err_msg);
```

Frontend uses `console.log/error`:

```typescript
console.log('[Home] Restored page state')
console.error('Failed to add download task:', error)
```

### Serialization

- Frontend: camelCase (TypeScript default)
- Backend: camelCase (`#[serde(rename_all = "camelCase")]`)

Example from `src-tauri/src/models/post.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GelbooruPost {
    pub id: u32,
    pub url: String,
    pub tag_list: Vec<GelbooruTag>,
    // ...
}
```

---

*Architecture analysis: 2026-04-13*
