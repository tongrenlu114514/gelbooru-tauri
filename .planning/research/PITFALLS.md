# Domain Pitfalls

**Domain:** Image Viewer + Tag Autocomplete + Download Management + Gallery Indexing
**Project:** Gelbooru Tauri Desktop App
**Analysis Date:** 2026-05-12
**Confidence:** HIGH (based on codebase analysis + debug history)

## Critical Pitfalls

### Pitfall 1: Base64 Memory Explosion with Large Images

**What goes wrong:** The current preview system loads images via `get_image_base64` which encodes the entire image as a base64 string. For a 10MB image, this creates a ~13MB JavaScript string in memory, plus the decoded image data. Multiple preview navigations accumulate this memory.

**Why it happens:**
```typescript
// Home.vue line 329 - current approach
const base64Url = await invoke<string>('get_image_base64', { url: previewUrl });
previewImageUrl.value = base64Url;  // Entire image as string
```

The `get_local_image_base64` command in gallery.rs reads the entire file into memory:
```rust
// gallery.rs line 335
let bytes = fs::read(&path_buf).map_err(...)?;
let base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
```

**Consequences:**
- Memory usage grows linearly with preview count
- Large images (>10MB) cause visible lag during decoding
- Browser tab may crash on extremely large images
- Memory is not released until navigation away

**Prevention:**
1. Use Tauri asset protocol for local images instead of base64:
   ```typescript
   import { convertFileSrc } from '@tauri-apps/api/core';
   const imageUrl = convertFileSrc(localPath);
   ```
2. For remote images, use streaming with range requests for preview
3. Implement image downscaling in backend for previews
4. Add explicit memory cleanup on preview close

**Detection:**
- Browser DevTools Memory tab shows increasing heap
- `console.log` of base64 string length shows >1MB warnings

---

### Pitfall 2: Unbounded Tag Dataset in Autocomplete

**What goes wrong:** Tag autocomplete loads ALL tags from search results into memory without pagination or filtering. Gelbooru can return thousands of tags per page.

**Why it happens:**
```typescript
// Current Home.vue stores all tags from every search
galleryStore.setTags(result.tagList);  // Line 271 - accumulates unbounded
```

The tag list from Gelbooru can contain:
- General tags (unbounded, often thousands)
- Artist tags
- Character tags
- Copyright tags
- Metadata tags

**Consequences:**
- Autocomplete dropdown renders thousands of items
- Search filter on large dataset causes UI lag
- Vue virtual list required but not implemented

**Prevention:**
1. Implement virtual scrolling for tag lists (use vue-virtual-scroller or similar)
2. Add debounced search with minimum character threshold (3+ characters)
3. Implement server-side tag caching with TTL
4. Separate autocomplete into type-specific searches

**Detection:**
- Tag list console.log shows >500 items
- Autocomplete dropdown takes >500ms to render

---

### Pitfall 3: Download State Desync on App Restart

**What goes wrong:** Download pause state is not persisted. When the app restarts with paused downloads, they lose their paused state and resume from pending.

**Why it happens:**
```rust
// download.rs - paused downloads are NOT marked in database
// Database only tracks: status, progress, sizes
// Pause position (byte offset) is not saved
```

The `pause_tokens` HashMap is in-memory only:
```rust
// download.rs line 89
pause_tokens: RwLock<HashMap<u32, mpsc::Sender<()>>>,  // Lost on restart
```

**Consequences:**
- User pauses download, closes app, reopens
- Download task is `pending` not `paused`
- User expects resume from byte offset, but starts from 0
- Wasted bandwidth re-downloading completed portion

**Prevention:**
1. Add `resume_offset` column to downloads table:
   ```sql
   ALTER TABLE downloads ADD COLUMN resume_offset INTEGER DEFAULT 0;
   ```
2. On pause, persist current byte offset to database
3. On resume, read offset and send Range header to server
4. Handle 206 Partial Content responses correctly

**Detection:**
- Compare downloaded file size before/after restart
- Check database `resume_offset` is null for paused tasks

---

### Pitfall 4: Thumbnail Generation Blocks Main Thread

**What goes wrong:** First-run gallery indexing generates thumbnails synchronously, blocking the UI for minutes on large galleries.

**Why it happens:**
```rust
// gallery.rs - thumbnail is just the FIRST image found
// No actual thumbnail generation
first_image: Option<String> = Some(path.to_string_lossy().to_string());
```

The current system uses the actual image path as "thumbnail" - no downscaling happens. Gallery.vue shows full-resolution images as thumbnails via `convertFileSrc`.

**Consequences:**
- Gallery navigation laggy during first load
- Memory pressure from multiple full-resolution images
- CLS (Cumulative Layout Shift) as images load at different sizes

**Prevention:**
1. Generate thumbnails in Rust backend using image crate:
   ```rust
   use image::GenericImageView;
   let thumb = img.thumbnail(200, 200);
   ```
2. Store thumbnails in `.thumbnail_cache/` subdirectory
3. Use async thumbnail generation with progress events
4. Implement lazy loading with intersection observer (already exists in GalleryCards.vue)

**Detection:**
- Gallery page freezes for >2 seconds on 1000+ images
- DevTools Performance tab shows long tasks

---

## Moderate Pitfalls

### Pitfall 5: Race Condition in Download Progress Events

**What goes wrong:** Download store and backend can get out of sync when multiple events arrive rapidly.

**Why it happens:**
```typescript
// download.ts line 114-127 - event handler
unlistenProgress = await listen<DownloadProgressEvent>('download-progress', (event) => {
  const data = event.payload;
  const index = tasks.value.findIndex((t) => t.id === data.id);
  if (index !== -1) {
    tasks.value[index] = { ...tasks.value[index], ...data };  // Race here
  }
});
```

If events arrive faster than Vue reactivity updates:
- `findIndex` may find wrong index
- Array splice during iteration causes issues

**Prevention:**
1. Use `Map<id, task>` instead of array for O(1) lookups
2. Add event sequence number for ordering
3. Debounce UI updates (100ms batches)

**Detection:**
- Occasionally wrong task shows progress
- Task list shows duplicate IDs temporarily

---

### Pitfall 6: Missing Error Boundary for Failed Image Loads

**What goes wrong:** Failed image loads in gallery/preview show nothing - no placeholder, no retry option, no error state.

**Why it happens:**
```vue
<!-- Home.vue line 552-555 - no error handling -->
<img
  :src="post.thumbnail || post.statistics.image"
  style="width: 100%; height: 100%; object-fit: cover"
/>
```

**Consequences:**
- Broken image icons from browser default
- No indication to user what went wrong
- No retry mechanism

**Prevention:**
1. Add `@error` handler to images
2. Show fallback placeholder image
3. Add retry button in preview modal
4. Log error for debugging

**Detection:**
- Broken image icons visible in gallery
- Failed downloads not visible to user

---

### Pitfall 7: Download Path Collision on Concurrent Same-Name Files

**What goes wrong:** Two downloads with same generated filename overwrite each other silently.

**Why it happens:**
```typescript
// download.ts line 93 - path generated but no uniqueness check
const savePath = `${basePath}/${dateStr}/${rating}/${copyright}/${fileName}`;
// fileName = `${characterPart}${meta.postId}${artistPart}.${ext}`
// postId collision possible if same image downloaded twice
```

**Consequences:**
- First download lost when second completes
- No indication of collision to user
- Partial files may remain

**Prevention:**
1. Use post ID + timestamp suffix:
   ```typescript
   const uniqueSuffix = `_${Date.now()}`;
   const fileName = `${characterPart}${meta.postId}${uniqueSuffix}${artistPart}.${ext}`;
   ```
2. Check for existing file before download
3. Prompt user for overwrite/rename

**Detection:**
- Completed downloads that are 0 bytes
- Missing expected downloads in folder

---

## Minor Pitfalls

### Pitfall 8: Preview Modal Scroll Locking

**What goes wrong:** Scrolling the page while preview is open scrolls content behind the modal.

**Why it happens:**
```vue
<!-- Home.vue - no scroll lock on modal open -->
<div v-if="showPreview" class="preview-overlay">
```

**Prevention:**
```css
.preview-overlay {
  overflow: hidden;  /* Add to CSS */
}
```

---

### Pitfall 9: Keyboard Navigation Not Disabled in Autocomplete

**What goes wrong:** ArrowUp/Down in tag input moves through autocomplete suggestions but also triggers browser back navigation.

**Why it happens:**
```vue
<!-- Home.vue line 524 - no keyboard capture -->
@n-input @keyup.enter="..."
```

**Prevention:**
- Prevent default on arrow keys when autocomplete is open
- Use `@keydown` instead of `@keyup`

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Image Viewer | Base64 memory explosion | Use asset protocol + lazy loading |
| Tag Autocomplete | Unbounded tag list | Virtual scrolling + debounced search |
| Download Pause/Resume | State not persisted | Add resume_offset to database |
| Thumbnail Generation | Blocks main thread | Background worker in Rust |
| Gallery Indexing | Slow initial load | Progressive loading + progress events |

## Sources

- Debug history: `.planning/debug/gallery-thumbnails-not-loading.md`
- Debug history: `.planning/debug/local-gallery-scroll-performance.md`
- Code review: `src/stores/download.ts`
- Code review: `src/views/Home.vue`
- Code review: `src-tauri/src/commands/gallery.rs`
- Code review: `src-tauri/src/commands/download.rs`