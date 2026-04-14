# Coding Conventions

**Analysis Date:** 2026-04-13

## Technology Stack

### Frontend
- **Framework:** Vue 3 with Composition API (`<script setup lang="ts">`)
- **State Management:** Pinia (Composition API pattern)
- **UI Library:** Naive UI
- **Router:** Vue Router 4 with lazy loading
- **TypeScript:** Strict mode enabled
- **Build Tool:** Vite 6
- **Path Alias:** `@/` maps to `src/`

### Backend
- **Framework:** Tauri 2.x (Rust)
- **Database:** SQLite via rusqlite (bundled)
- **Async Runtime:** tokio (full features)
- **HTTP Client:** reqwest (with cookies, gzip, streaming)
- **HTML Parsing:** scraper
- **Error Handling:** thiserror

---

## TypeScript Conventions

### Type Definitions

**Location:** `src/types/index.ts`

All shared TypeScript interfaces are defined in `src/types/index.ts`:

```typescript
// Interfaces are PascalCase
export interface GelbooruPost {
  id: number
  url: string
  title: string
  tagList: GelbooruTag[]
  statistics: GelbooruPostStatistics
  thumbnail?: string
}

// Union types for status literals
export interface DownloadTask {
  status: 'pending' | 'downloading' | 'completed' | 'failed' | 'paused' | 'cancelled'
}
```

### Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Files | kebab-case | `gallery-store.ts` |
| Interfaces | PascalCase | `GelbooruPost` |
| Types | PascalCase | `ImageInfo` |
| Variables | camelCase | `searchTags` |
| Functions | camelCase | `searchPosts()` |
| Constants | camelCase or UPPER_SNAKE_CASE | `PAGE_SIZE` or `pageSize` |
| Vue Components | PascalCase | `Gallery.vue` |
| Props | camelCase | `imageCount` |

### Import Organization

**Order:**
1. Vue core imports (`vue`)
2. Vue Router imports (`vue-router`)
3. Third-party library imports (naive-ui, pinia)
4. Custom types (`@/types`)
5. Custom stores (`@/stores`)
6. Tauri API imports (`@tauri-apps/api/core`)
7. Relative imports (`./`, `../`)

```typescript
// Example from src/views/Home.vue
import { ref, onMounted, watch, computed, nextTick } from 'vue'
import { onBeforeRouteLeave, useRoute } from 'vue-router'
import {
  NInput, NButton, NSpace, NTag,
  useMessage
} from 'naive-ui'
import { useGalleryStore } from '@/stores/gallery'
import { useDownloadStore } from '@/stores/download'
import { invoke } from '@tauri-apps/api/core'
import type { GelbooruPost, GelbooruTag } from '@/types'
```

### Vue Component Structure

**Script Setup Pattern:**

```vue
<script setup lang="ts">
// 1. Imports
import { ref, computed, onMounted } from 'vue'
import { NButton, useMessage } from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'

// 2. Store usage
const store = useMyStore()
const message = useMessage()

// 3. Local state (camelCase, with type annotations)
const isLoading = ref(false)
const selectedId = ref<number | null>(null)

// 4. Computed properties
const filteredItems = computed(() => ...)

// 5. Functions (camelCase)
function handleSelect(id: number) {
  // ...
}

// 6. Lifecycle hooks
onMounted(() => {
  // ...
})
</script>

<template>
  <!-- Template content -->
</template>

<style scoped>
/* Scoped styles */
</style>
```

### Pinia Store Pattern

**Location:** `src/stores/*.ts`

Use Composition API style with `defineStore`:

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { MyData } from '@/types'

export const useMyStore = defineStore('myStore', () => {
  // State as refs
  const items = ref<MyData[]>([])
  const loading = ref(false)

  // Computed
  const activeItems = computed(() =>
    items.value.filter(i => i.status === 'active')
  )

  // Actions
  async function fetchItems() {
    loading.value = true
    try {
      const result = await invoke<MyData[]>('my_command')
      items.value = result
    } catch (error) {
      console.error('Failed to fetch:', error)
    } finally {
      loading.value = false
    }
  }

  // Return public API
  return {
    items,
    loading,
    activeItems,
    fetchItems
  }
})
```

### Error Handling

**Frontend (TypeScript):**
```typescript
async function searchPosts() {
  loading.value = true
  try {
    const result = await invoke<SearchResult>('search_posts', { tags })
    // Handle success
  } catch (error) {
    console.error('Search failed:', error)
    // Show user-friendly message
  } finally {
    loading.value = false
  }
}
```

**Backend (Rust):**
- Return `Result<T, String>` for Tauri commands
- Use `?` operator for propagation
- Map errors to user-friendly messages

---

## Rust Conventions

### File Organization

```
src-tauri/src/
├── main.rs           # Entry point, Tauri builder setup
├── lib.rs             # Module declarations
├── commands/          # Tauri command handlers
│   ├── mod.rs
│   ├── gelbooru.rs
│   ├── download.rs
│   ├── gallery.rs
│   └── favorite_tags.rs
├── models/            # Data structures
│   ├── mod.rs
│   ├── post.rs
│   ├── tag.rs
│   └── page.rs
├── services/          # Business logic
│   ├── mod.rs
│   ├── scraper.rs
│   └── http.rs
└── db/                # Database layer
    └── mod.rs
```

### Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Modules | snake_case | `src-tauri/src/db/` |
| Structs | PascalCase | `GelbooruPost` |
| Fields | snake_case | `post_id`, `image_url` |
| JSON fields | camelCase | `#[serde(rename_all = "camelCase")]` |
| Functions | snake_case | `get_post_detail()` |
| Constants | SCREAMING_SNAKE_CASE | `PAGE_SIZE` |
| Traits | PascalCase | `Serialize` |

### Data Structures

**Models with serde:**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GelbooruPost {
    pub id: u32,
    pub url: String,
    pub title: String,
    pub tag_list: Vec<GelbooruTag>,
    pub statistics: GelbooruPostStatistics,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
}
```

### Tauri Commands

```rust
use tauri::command;

#[tauri::command]
pub async fn search_posts(
    tags: Vec<String>,
    page: u32,
) -> Result<SearchResult, String> {
    // Validate input
    if tags.is_empty() {
        return Err("Tags cannot be empty".to_string());
    }

    // Execute with proper error mapping
    do_search(tags, page).await.map_err(|e| e.to_string())
}
```

### Async Patterns

- Use `tokio::task::spawn_blocking` for CPU-intensive operations
- Use `.map_err(|e| e.to_string())` for Tauri command error conversion
- Pass data to blocking tasks via `move` closures

```rust
#[tauri::command]
pub async fn get_directory_tree(folder_path: Option<String>) -> Result<Vec<TreeNode>, String> {
    let path_str = folder_path.unwrap_or_else(|| default_path.to_string());

    let result = tokio::task::spawn_blocking(move || {
        // Blocking file system operations
        build_tree_from_path(&path_str)
    }).await.map_err(|e| e.to_string())?;

    Ok(result)
}
```

### Database Pattern

**Location:** `src-tauri/src/db/mod.rs`

- Use `Mutex<Connection>` for thread-safe access
- Use rusqlite parameterized queries to prevent SQL injection
- Return typed results

```rust
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn add_favorite_tag(&self, tag: &str, tag_type: &str) -> SqliteResult<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO favorite_tags (tag, tag_type) VALUES (?1, ?2)",
            rusqlite::params![tag, tag_type],
        )?;
        Ok(conn.last_insert_rowid())
    }
}
```

---

## Component Patterns

### File Size Guidelines

Per user instruction: **Each file should not exceed 200 lines**

- Large components should be split into smaller, reusable pieces
- Extract complex logic into composables or utilities
- Extract reusable UI sections into child components

### Scoped Styles

```vue
<style scoped>
.home-view {
  padding: 0;
}

.post-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 12px;
}

.post-card {
  border-radius: 8px;
  overflow: hidden;
  cursor: pointer;
}
</style>
```

### Event Handling

```vue
<!-- Native events -->
<button @click="handleClick">Click</button>

<!-- With event modifier -->
<button @click.stop="handleClick">Stop propagation</button>

<!-- With keyboard -->
<input @keyup.enter="handleSubmit" />

<!-- Stop propagation for nested handlers -->
<n-button @click.stop="handleAction">
```

---

## State Management Patterns

### Store Responsibilities

| Store | File | Responsibilities |
|-------|------|------------------|
| Gallery | `src/stores/gallery.ts` | Search results, pagination, page state |
| Download | `src/stores/download.ts` | Download tasks, queue management |
| Settings | `src/stores/settings.ts` | App configuration, theme |
| Favorite Tags | `src/stores/favoriteTags.ts` | Tag favorites management |

### Cross-Store Communication

```typescript
// From download.ts - accessing settings store
import { useSettingsStore } from './settings'

async function addTask(meta: PostMeta) {
  const settingsStore = useSettingsStore()
  const savePath = generateSavePath(meta, settingsStore.downloadPath)
  // ...
}
```

---

## API Communication

### Tauri Invoke Pattern

**Calling Rust commands from TypeScript:**

```typescript
import { invoke } from '@tauri-apps/api/core'

// Simple call
const result = await invoke<ReturnType>('command_name', { param1, param2 })

// With error handling
try {
  const result = await invoke<ReturnType>('command_name', { param1 })
} catch (error) {
  console.error('Command failed:', error)
}
```

**Tauri Event Listening:**

```typescript
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

let unlisten: UnlistenFn | null = null

async function init() {
  unlisten = await listen<DownloadProgressEvent>('download-progress', (event) => {
    // Handle progress
  })
}

// Clean up on unmount
onUnmounted(() => {
  if (unlisten) unlisten()
})
```

---

## CSS Conventions

### Class Naming

- Use kebab-case for CSS class names
- Use semantic class names based on component/function

### CSS Organization in Components

```vue
<style scoped>
/* 1. Layout */
.container { }

/* 2. Components */
.card { }
.button { }

/* 3. States */
.is-loading { }
.is-active { }

/* 4. Utilities */
.text-center { }
</style>
```

---

*Convention analysis: 2026-04-13*
