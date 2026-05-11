# Architecture Patterns for New Features

**Research Date:** 2026-05-12
**Project:** gelbooru (Tauri Vue Desktop App)
**Domain:** Image viewer, tag autocomplete, download pause/resume, gallery indexing

## Executive Summary

The existing architecture provides strong patterns for implementing the new features. Image viewer already exists as a modal pattern; tag autocomplete requires a new composable; download pause/resume is fully implemented; gallery indexing needs background task infrastructure.

## 1. Image Viewer Integration

### Current Pattern (EXISTING)

Both `Home.vue` and `Gallery.vue` implement image preview modals:

```
src/views/Home.vue:578-718        (overlay modal, keyboard nav, tag sidebar)
src/views/Gallery.vue:296-343     (NModal with prev/next/delete)
```

**Pattern established:**
- Overlay modal via `v-if="showPreview"` or `NModal v-model:show`
- Full-screen container with `position: fixed`
- Keyboard navigation via `window.addEventListener('keydown', handler)`
- Image src via `convertFileSrc()` for local files, `get_image_base64` for remote

### Integration Decision: Reuse Modal Pattern

**Recommend: Component-based modal (NOT new route)**

Rationale:
- Faster navigation (no route transition delay)
- State preserved across views (same page scroll position)
- Consistent UX with existing preview behavior
- Gallery.vue already uses this pattern successfully

### New Component Structure

```
src/components/ImageViewer.vue     # Reusable viewer modal
src/composables/useImageViewer.ts  # State management composable
```

**New vs Modified:**

| Component | Action | Purpose |
|-----------|--------|---------|
| `ImageViewer.vue` | NEW | Reusable modal with keyboard nav, zoom, tags |
| `useImageViewer.ts` | NEW | Composable managing preview state |
| `Home.vue` | MODIFY | Replace inline preview with ImageViewer |
| `Gallery.vue` | MODIFY | Replace inline preview with ImageViewer |

**Data Flow:**

```
User clicks thumbnail
    │
    ▼
useImageViewer.open(index, postList)
    │
    ├── Set currentPost, showPreview
    ├── invoke('get_post_detail') for full data
    ├── invoke('get_image_base64') for image
    ▼
<ImageViewer :visible="show" :post="currentPost" @close="close" />
    │
    ├── ArrowLeft/Right → navigate posts
    ├── Escape → close
    ├── Download button → downloadStore.addTask()
    ├── Tag click → search with tag
```

## 2. Tag Autocomplete Integration

### Current State

No autocomplete exists. Tag input in `Home.vue` (line 520-527):
```typescript
<n-input v-model:value="searchInput" placeholder="输入标签搜索..." @keyup.enter="addTag(searchInput)" />
```

### Integration Decision: Frontend-Only with Backend Hint Cache

**Recommend: Frontend autocomplete + backend hint caching**

Rationale:
- Tag suggestions come from Gelbooru search results (already fetched)
- Latency-sensitive: autocomplete must be fast (<100ms)
- Backend hints can be cached locally (Redis not needed, just in-memory + SQLite)

### Architecture

```
src/composables/useTagAutocomplete.ts   # Autocomplete logic
src-tauri/src/commands/tags.rs           # NEW: tag suggestion commands
```

**New vs Modified:**

| Component | Action | Purpose |
|-----------|--------|---------|
| `useTagAutocomplete.ts` | NEW | Debounced search, LRU cache, gelbooru suggestion merge |
| `commands/tags.rs` | NEW | Backend tag hint fetching |
| `Home.vue` | MODIFY | Integrate with NAutoComplete |
| `db/mod.rs` | MODIFY | Add tag_hints table |

**Tag Hint Storage (SQLite):**

```sql
CREATE TABLE tag_hints (
    tag TEXT PRIMARY KEY,
    tag_type TEXT,
    usage_count INTEGER DEFAULT 0,
    last_used TIMESTAMP
);
```

**Cache Strategy:**
- LRU cache in frontend composable (50-100 tags)
- SQLite for persistence across sessions
- Backend fetches from Gelbooru on cache miss
- Merge: user history > cached hints > live search

**Data Flow:**

```
User types in search input
    │
    ▼
useTagAutocomplete.search(query)
    │
    ├── 1. Check user history (exact match)
    ├── 2. Check LRU cache
    ├── 3. If cache miss → invoke('get_tag_hints')
    │       │
    │       ▼
    │   Backend checks SQLite cache
    │       │
    │       ▼
    │   If stale → fetch from Gelbooru, update cache
    │       │
    │       ▼
    │   Return suggestions
    │
    ▼
N-auto-complete shows suggestions
```

## 3. Download Pause/Resume Integration

### Current Implementation (EXISTING)

**Fully implemented in `src-tauri/src/commands/download.rs`:**

- Line 83-90: `DownloadManager` with `cancel_tokens` and `pause_tokens`
- Line 530-543: `pause_download` command (sends signal via mpsc channel)
- Line 545-552: `resume_download` command (reuses start_download)
- Line 379-400: Pause detection during download loop

**State persistence:**
- Line 219-236: `save_download_task()` to SQLite on task creation
- Line 361-388: `save_download_task()` with ON CONFLICT for updates
- Line 642-666: Async persistence helpers for spawned tasks

### Integration Decision: No Changes Needed

**Existing implementation is sufficient.**

The pause/resume pattern already:
- Stores partial download in `.tmp` file
- Resumes from last byte position (content-length aware)
- Persists state to SQLite
- Emits progress events for UI updates

**What exists vs what's needed:**

| Concern | Status | Implementation |
|---------|--------|----------------|
| Pause signal channel | EXISTS | `pause_tokens: RwLock<HashMap<u32, mpsc::Sender<()>>>` |
| Resume logic | EXISTS | `resume_download` → `start_download` |
| Partial file retention | EXISTS | Line 350: temp file with `.tmp` extension |
| State persistence | EXISTS | SQLite via `save_download_task()` |
| UI integration | EXISTS | Pinia store + event listener |

**No new components needed.** Interface already exposed.

## 4. Gallery Index Integration

### Current State

**Gallery already has directory tree and image listing:**
- `commands/gallery.rs`: `get_directory_tree`, `get_directory_images`
- `Gallery.vue`: Tree navigation, image grid, preview modal
- Images collected via `walkdir::WalkDir` recursively

**Current flow (gallery.rs:623-680):**
```
get_directory_images_async()
    │
    ├── collect_images_recursive() via WalkDir
    ├── Sort by mtime descending
    ├── Paginate with offset/limit
    ▼
Return DirectoryImages { subdirs, images, total, has_more, offset, limit }
```

### Integration Decision: Hybrid Approach (Background + On-Demand)

**Recommend: SQLite index table + on-demand refresh**

**Why hybrid?**
- Directory tree already exists and works
- Background indexing needed for large libraries (10K+ images)
- On-demand ensures accuracy without stale index
- Index speeds up initial gallery load

### New Components

```
src-tauri/src/commands/indexer.rs    # Background indexer commands
src/composables/useGalleryIndex.ts   # Frontend index state
```

**Database Schema:**

```sql
CREATE TABLE gallery_index (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    filename TEXT NOT NULL,
    file_size INTEGER,
    mtime INTEGER,
    indexed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_gallery_index_path ON gallery_index(path);
CREATE INDEX idx_gallery_index_mtime ON gallery_index(mtime DESC);
```

**Indexer Architecture:**

```rust
// src-tauri/src/commands/indexer.rs

pub struct Indexer {
    tasks: RwLock<HashMap<PathBuf, IndexTask>>,
    semaphore: Semaphore,
}

#[tauri::command]
pub async fn start_indexing(dir_path: String) -> Result<(), String> {
    // Spawn background task to walk directory and update SQLite
    // Emit 'index-progress' events to frontend
}

#[tauri::command]
pub async fn get_index_status(dir_path: String) -> Result<IndexStatus, String> {
    // Return indexed count, pending count, last indexed time
}

#[tauri::command]
pub async fn query_index(query: String, limit: usize) -> Result<Vec<IndexedImage>, String> {
    // Fast index search (path LIKE '%query%' OR filename LIKE '%query%')
}
```

**Build Order (Dependencies):**

```
Phase 1: Index infrastructure
    ├── db/mod.rs: Add gallery_index table
    ├── commands/indexer.rs: Basic CRUD + walk
    └── composables/useGalleryIndex.ts: Frontend state

Phase 2: Background indexing
    ├── Background task spawning
    ├── Progress events
    └── Index status queries

Phase 3: Gallery integration
    ├── Gallery.vue: Use index for fast load
    ├── Fallback to on-demand if index stale
    └── Manual refresh trigger
```

## Component Summary

| Feature | New Components | Modified Components | Notes |
|---------|----------------|---------------------|-------|
| Image Viewer | `ImageViewer.vue`, `useImageViewer.ts` | `Home.vue`, `Gallery.vue` | Reuse existing modal pattern |
| Tag Autocomplete | `useTagAutocomplete.ts`, `commands/tags.rs` | `Home.vue`, `db/mod.rs` | LRU cache + SQLite hints |
| Download Pause/Resume | NONE | NONE | Fully implemented |
| Gallery Index | `commands/indexer.rs`, `useGalleryIndex.ts` | `Gallery.vue`, `db/mod.rs` | Hybrid background/on-demand |

## Integration Points

### Pinia Store Extension

```typescript
// src/stores/gallery.ts - Add index state
export const useGalleryStore = defineStore('gallery', () => {
  // ... existing state ...

  // New: index state
  const indexStatus = ref<'idle' | 'indexing' | 'ready'>('idle');
  const indexedCount = ref(0);

  // ... existing functions ...
});
```

### IPC Commands to Add

```rust
// src-tauri/src/commands/mod.rs - Register new modules
pub mod tags;      // Tag autocomplete hints
pub mod indexer;   // Gallery background indexing
```

### Event Channels

```rust
// New events to emit
"index-progress"     // { dir: string, indexed: number, total: number }
"index-complete"     // { dir: string, count: number }
"tag-hints-ready"    // { query: string, hints: TagHint[] }
```

## Validation Against Existing Patterns

### Pattern 1: Pinia Store with Event Listeners

From `src/stores/download.ts:110-128`:
```typescript
async function initListeners() {
  unlistenProgress = await listen<DownloadProgressEvent>('download-progress', (event) => {
    // Update store from event
  })
}
```

**Follow this pattern for:**
- `useGalleryIndex.ts`: Listen to `index-progress` events
- `useTagAutocomplete.ts`: Listen to `tag-hints-ready` events

### Pattern 2: Command Registration

From `src-tauri/src/main.rs:200-229`:
```rust
invoke_handler(tauri::generate_handler![
    commands::gelbooru::search_posts,
    commands::download::start_download,
    // ...
])
```

**Follow this pattern for:**
- `commands::tags::get_tag_hints`
- `commands::indexer::start_indexing`
- `commands::indexer::get_index_status`

### Pattern 3: Database State Pattern

From `src-tauri/src/commands/favorite_tags.rs:DbState`:
```rust
pub struct DbState(pub Arc<Mutex<Database>>);

// Used in commands:
#[tauri::command]
pub async fn add_parent_tag(db: State<'_, DbState>, ...) -> Result<(), String> {
    let database = db.0.lock().map_err(|e| e.to_string())?;
    // ...
}
```

**Follow this pattern for:**
- `commands::tags`: DbState for tag_hints table
- `commands::indexer`: DbState for gallery_index table

## Risk Assessment

| Feature | Risk | Mitigation |
|---------|------|------------|
| Image Viewer reuse | Low | Pattern already tested in Home.vue and Gallery.vue |
| Tag autocomplete cache | Medium | LRU cache prevents unbounded growth; SQLite TTL for persistence |
| Download pause/resume | None | Fully implemented and tested |
| Gallery index performance | Medium | Semaphore limits concurrent file operations; chunked updates |

## Confidence

- **HIGH** for Image Viewer (existing pattern)
- **HIGH** for Download Pause/Resume (already implemented)
- **MEDIUM** for Tag Autocomplete (pattern clear, needs careful cache sizing)
- **MEDIUM** for Gallery Index (hybrid approach adds complexity)

---

*Research complete. Integration points identified for all four features.*