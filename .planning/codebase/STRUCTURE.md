# Codebase Structure

**Analysis Date:** 2026-04-13

## Directory Layout

```
gelbooru/
├── src/                          # Vue.js Frontend
│   ├── main.ts                   # App entry point
│   ├── App.vue                   # Root component
│   ├── vite-env.d.ts            # Vite type declarations
│   ├── router/
│   │   └── index.ts             # Vue Router configuration
│   ├── stores/
│   │   ├── gallery.ts           # Gallery state (posts, tags, pagination)
│   │   ├── download.ts          # Download tasks state
│   │   ├── settings.ts          # App settings state
│   │   └── favoriteTags.ts      # Favorite tags state
│   ├── types/
│   │   └── index.ts             # TypeScript interfaces
│   ├── views/
│   │   ├── Home.vue             # Search and post grid
│   │   ├── Gallery.vue          # Local file browser
│   │   ├── Downloads.vue        # Download task manager
│   │   ├── Settings.vue         # App settings
│   │   └── FavoriteTags.vue     # Favorite tag manager
│   └── components/
│       ├── AppSidebar.vue       # Navigation sidebar
│       └── DownloadNotifier.vue # Download notification listener
│
├── src-tauri/                    # Rust Backend (Tauri)
│   ├── src/
│   │   ├── main.rs             # Tauri app entry, command registration
│   │   ├── lib.rs              # Module declarations (empty)
│   │   ├── commands/           # Tauri command handlers
│   │   │   ├── mod.rs          # Module exports
│   │   │   ├── gelbooru.rs     # Search and scraping commands
│   │   │   ├── download.rs     # Download management commands
│   │   │   ├── gallery.rs      # Local file commands
│   │   │   └── favorite_tags.rs # Favorite tag CRUD commands
│   │   ├── services/           # Business logic
│   │   │   ├── mod.rs          # Module exports
│   │   │   ├── http.rs         # HTTP client with proxy support
│   │   │   └── scraper.rs      # HTML parsing service
│   │   ├── models/             # Data structures
│   │   │   ├── mod.rs          # Module exports
│   │   │   ├── post.rs         # GelbooruPost, GelbooruPostStatistics
│   │   │   ├── tag.rs          # GelbooruTag
│   │   │   └── page.rs         # GelbooruPage
│   │   └── db/
│   │       └── mod.rs          # SQLite database layer
│   ├── tauri.conf.json         # Tauri configuration
│   ├── Cargo.toml              # Rust dependencies
│   ├── capabilities/           # Tauri permissions
│   │   └── default.json
│   ├── gen/                    # Generated Tauri schemas
│   └── icons/                  # App icons
│
├── cookie/                      # Cookie storage (for Gelbooru auth)
├── dist/                        # Build output
├── node_modules/               # NPM dependencies
├── package.json               # Node.js dependencies
├── tsconfig.json              # TypeScript configuration
├── vite.config.ts             # Vite build configuration
└── .planning/codebase/        # Architecture documentation
```

## Directory Purposes

### Frontend Source (`src/`)

**Purpose:** Vue.js single-page application

**Key files:**
- `main.ts`: App initialization (Pinia, Router, NaiveUI)
- `App.vue`: Root component with layout (sidebar, header, content)

### Frontend Routing (`src/router/`)

**Purpose:** SPA route configuration

**File:** `src/router/index.ts`

**Routes:**
| Path | Name | Component | Purpose |
|------|------|-----------|---------|
| `/` | home | `Home.vue` | Search posts, tag filtering |
| `/downloads` | downloads | `Downloads.vue` | Download task management |
| `/gallery` | gallery | `Gallery.vue` | Local image browser |
| `/favorite-tags` | favorite-tags | `FavoriteTags.vue` | Tag favorites management |
| `/settings` | settings | `Settings.vue` | App configuration |

**Pattern:** Lazy-loaded routes with `() => import('@/views/...')`

### Frontend State (`src/stores/`)

**Purpose:** Pinia stores for reactive state management

**Files and responsibilities:**

| Store | State | Key Actions |
|-------|-------|-------------|
| `gallery.ts` | posts, tags, pagination, pageState | setPosts, setTags, savePageState, restorePageState |
| `download.ts` | tasks, isDownloading | addTask, startDownload, pauseDownload, resumeDownload, cancelDownload |
| `settings.ts` | theme, downloadPath, proxyConfig | toggleTheme, toggleSidebar, getProxyUrl |
| `favoriteTags.ts` | tags, loading | loadTags, addParentTag, addChildTag, removeTag, findTagGroup |

### Frontend Views (`src/views/`)

**Purpose:** Page-level components

**Files:**

| File | Lines | Purpose |
|------|-------|---------|
| `Home.vue` | ~980 | Post search, grid display, preview modal, tag management |
| `Gallery.vue` | ~610 | Directory tree, image grid, preview modal |
| `Downloads.vue` | ~200 | DataTable with download tasks |
| `Settings.vue` | ~110 | Form for app settings |
| `FavoriteTags.vue` | ~400 | Tag group management with collapse |

### Frontend Components (`src/components/`)

**Purpose:** Reusable UI components

| Component | Purpose |
|-----------|---------|
| `AppSidebar.vue` | Navigation menu with collapsible sidebar |
| `DownloadNotifier.vue` | Non-rendering component that listens to download events |

### Frontend Types (`src/types/`)

**Purpose:** TypeScript interfaces for frontend-backend communication

**Location:** `src/types/index.ts`

**Key interfaces:**
```typescript
interface GelbooruPost {
  id: number
  url: string
  title: string
  tagList: GelbooruTag[]
  statistics: GelbooruPostStatistics
  thumbnail?: string
}

interface GelbooruTag {
  text: string
  tagType: string
  count: number
}

interface DownloadTask {
  id: number
  postId: number
  imageUrl: string
  fileName: string
  savePath: string
  status: 'pending' | 'downloading' | 'completed' | 'failed' | 'paused' | 'cancelled'
  progress: number
  // ...
}
```

### Backend Commands (`src-tauri/src/commands/`)

**Purpose:** Tauri command handlers (frontend-backend bridge)

**Files and commands:**

| File | Commands | Purpose |
|------|----------|---------|
| `gelbooru.rs` | search_posts, get_post_detail, get_image_base64, set_proxy | Gelbooru API integration |
| `download.rs` | add_download_task, start_download, pause_download, resume_download, cancel_download, remove_download_task, get_download_tasks, clear_completed_tasks, open_file | Download management |
| `gallery.rs` | get_local_images, delete_image, get_directory_tree, get_directory_images, get_local_image_base64 | Local file operations |
| `favorite_tags.rs` | get_favorite_tags, add_parent_tag, add_child_tag, remove_favorite_tag, is_tag_favorited, get_child_tags | Tag favorites CRUD |

### Backend Services (`src-tauri/src/services/`)

**Purpose:** Business logic and external integrations

**Files:**

| File | Purpose |
|------|---------|
| `http.rs` | HTTP client wrapper with proxy, cookies, referer support |
| `scraper.rs` | HTML parsing for Gelbooru pages (post list, post detail, tags) |

### Backend Models (`src-tauri/src/models/`)

**Purpose:** Data structures

**Files:**

| File | Structures |
|------|------------|
| `post.rs` | GelbooruPost, GelbooruPostStatistics |
| `tag.rs` | GelbooruTag |
| `page.rs` | GelbooruPage |

### Backend Database (`src-tauri/src/db/`)

**Purpose:** SQLite persistence

**Tables:**
- `downloads`: Download history and progress
- `favorites`: Favorited posts
- `blacklisted_tags`: Blocked tags
- `favorite_tags`: Hierarchical favorite tags with parent-child relationships

## Key File Locations

### Entry Points

| Purpose | Location |
|---------|----------|
| Frontend main | `src/main.ts` |
| Frontend root | `src/App.vue` |
| Backend main | `src-tauri/src/main.rs` |

### Configuration

| Purpose | Location |
|---------|----------|
| Tauri config | `src-tauri/tauri.conf.json` |
| Vite config | `vite.config.ts` |
| TypeScript config | `tsconfig.json` |
| NPM dependencies | `package.json` |
| Rust dependencies | `src-tauri/Cargo.toml` |

### Routing

| Purpose | Location |
|---------|----------|
| Router config | `src/router/index.ts` |

### State Management

| Store | Location |
|-------|----------|
| Gallery state | `src/stores/gallery.ts` |
| Download state | `src/stores/download.ts` |
| Settings state | `src/stores/settings.ts` |
| Favorite tags state | `src/stores/favoriteTags.ts` |

### Tauri Commands

| Command Group | Location |
|---------------|----------|
| Gelbooru commands | `src-tauri/src/commands/gelbooru.rs` |
| Download commands | `src-tauri/src/commands/download.rs` |
| Gallery commands | `src-tauri/src/commands/gallery.rs` |
| Favorite tags commands | `src-tauri/src/commands/favorite_tags.rs` |

## Naming Conventions

### Files

| Type | Pattern | Example |
|------|---------|---------|
| Vue components | PascalCase | `Home.vue`, `AppSidebar.vue` |
| TypeScript modules | camelCase | `gallery.ts`, `favoriteTags.ts` |
| Rust modules | snake_case | `scraper.rs`, `favorite_tags.rs` |
| Rust commands | snake_case | `search_posts`, `get_post_detail` |

### Directories

| Type | Pattern | Example |
|------|---------|---------|
| Vue directories | kebab-case | `src/views/`, `src/components/` |
| Rust directories | snake_case | `src-tauri/src/commands/` |

### Vue Components

- Single-file components (`.vue`)
- `<script setup lang="ts">` syntax
- Scoped styles with `<style scoped>`

### Pinia Stores

```typescript
// Naming: use{StoreName}Store
export const useGalleryStore = defineStore('gallery', () => {
  // Store implementation
})
```

### TypeScript Types

```typescript
// Interfaces: PascalCase
interface GelbooruPost { ... }

// Enums: PascalCase
type DownloadStatus = 'pending' | 'downloading' | 'completed' | ...

// Props: camelCase
interface Props {
  isLoading: boolean
  title: string
}
```

### Rust

```rust
// Structs: PascalCase
pub struct GelbooruPost { ... }

// Functions: snake_case
pub async fn search_posts(tags: Vec<String>, page: u32) -> Result<...> { ... }

// Modules: snake_case
pub mod favorite_tags;

// Enums: PascalCase
pub enum DownloadStatus { ... }
```

## Component Hierarchy

```
App.vue (Root)
├── NLayout (Main layout)
│   ├── NLayoutSider
│   │   └── AppSidebar.vue
│   └── NLayout
│       ├── NLayoutHeader
│       └── NLayoutContent
│           └── RouterView
│               ├── Home.vue
│               │   ├── Post grid
│               │   ├── Search bar
│               │   ├── Tag selector
│               │   └── Preview modal
│               ├── Gallery.vue
│               │   ├── Directory tree
│               │   └── Image grid + preview
│               ├── Downloads.vue
│               │   └── DataTable with actions
│               ├── Settings.vue
│               │   └── Settings form
│               └── FavoriteTags.vue
│                   └── Collapse accordion
└── DownloadNotifier.vue (Non-rendering, event listener)
```

## Routing Structure

```
/                           → Home.vue (Search posts)
/downloads                   → Downloads.vue (Download manager)
/gallery                     → Gallery.vue (Local browser)
/favorite-tags               → FavoriteTags.vue (Tag favorites)
/settings                    → Settings.vue (App settings)
```

### Route Guards

```typescript
// Home.vue uses onBeforeRouteLeave
onBeforeRouteLeave(() => {
  galleryStore.savePageState(selectedTags.value, selectedRating.value)
})
```

### Query Parameters

```typescript
// Home.vue reads query for tag from FavoriteTags.vue
onMounted(() => {
  if (route.query.tag) {
    selectedTags.value.push(route.query.tag as string)
    searchPosts(true)
  }
})

// FavoriteTags.vue navigates with query
function searchWithTag(tag: string) {
  router.push({ name: 'home', query: { tag } })
}
```

## Module Organization

### Frontend Modules

```
src/
├── main.ts           # Bootstrap
├── App.vue           # Root component
├── router/           # Routing
├── stores/           # State management
├── views/            # Pages
├── components/       # Shared components
└── types/            # Type definitions
```

### Backend Modules

```
src-tauri/src/
├── main.rs           # Entry point + command registration
├── lib.rs            # Module declarations
├── commands/         # Tauri command handlers
├── services/         # Business logic
├── models/           # Data structures
└── db/               # Persistence
```

## Where to Add New Code

### New Feature (Frontend + Backend)

1. **Types:** Add interface to `src/types/index.ts`
2. **Backend command:** Add to `src-tauri/src/commands/`
3. **Backend service:** Add to `src-tauri/src/services/` if needed
4. **Register command:** Update `src-tauri/src/main.rs`
5. **Frontend store:** Add to `src/stores/`
6. **Frontend view:** Add to `src/views/`
7. **Route:** Update `src/router/index.ts`

### New Component

1. Place in `src/components/` following naming convention
2. Use `<script setup lang="ts">` syntax
3. Add scoped styles
4. Import in parent view

### New Tauri Command

1. Add to appropriate file in `src-tauri/src/commands/`
2. Use `#[tauri::command]` attribute
3. Return `Result<T, String>` for error handling
4. Register in `src-tauri/src/main.rs` `invoke_handler`

### New Pinia Store

1. Create `src/stores/{feature}.ts`
2. Use setup store pattern with `defineStore`
3. Export composable: `export const use{Feature}Store`
4. Import in views as needed

### New Rust Model

1. Add to appropriate file in `src-tauri/src/models/`
2. Add `#[derive(Serialize, Deserialize)]`
3. Add `#[serde(rename_all = "camelCase")]`
4. Export from `src-tauri/src/models/mod.rs`

## Special Directories

### Cookie Storage (`cookie/`)

**Purpose:** Store Gelbooru cookies for authentication

**Format:** JSON files per domain

**Usage:** Loaded by `HttpClient::load_cookies()`

### Generated Files (`src-tauri/gen/`)

**Purpose:** Auto-generated Tauri schema files

**Status:** Generated, not committed to version control

### Build Output (`dist/`)

**Purpose:** Vite build output

**Status:** Generated, not committed

---

*Structure analysis: 2026-04-13*
