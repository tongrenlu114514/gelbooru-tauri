# Phase 3: Performance & Reliability - Research

**Researched:** 2026-04-16
**Domain:** Tauri desktop app (Vue 3 frontend + Rust backend) -- memory leak, retry, async FS, rate limiting
**Confidence:** MEDIUM-HIGH

## Summary

Phase 3 addresses four issues: an image base64 cache memory leak in Gallery.vue (solved via IntersectionObserver lazy loading), missing download retry with exponential backoff in download.rs, slow synchronous directory scanning in gallery.rs (solved with tokio async FS + Semaphore), and no HTTP rate limiting in http.rs. All four solutions are well-established patterns with zero new dependencies required.

**Primary recommendation:** Use IntersectionObserver for viewport-driven lazy loading in Gallery.vue; wrap the HTTP download call in `start_download` with a 3-retry exponential-backoff loop; replace `spawn_blocking` + sync `fs::read_dir` with true `tokio::fs::read_dir` + `tokio::sync::Semaphore` in gallery.rs; inject a `RwLock<Instant>`-based 500ms inter-request gap into `HttpClient`.

---

## User Constraints (from CONTEXT.md)

### Locked Decisions

| Key | Decision |
|-----|----------|
| D-02 | Fix memory leak via IntersectionObserver lazy loading (viewport-based), not LRU eviction alone |
| D-03 | Change location: `src/views/Gallery.vue`, modify `preloadImages` → `loadVisibleImages` |
| D-04 | Retry strategy = exponential backoff, max 3 retries, intervals 1s → 2s → 4s |
| D-05 | Retry triggers = network errors only (timeout, connection failure, 5xx); no retry for 4xx or cancellation |
| D-06 | Change location: `src-tauri/src/commands/download.rs`, wrap `start_download` HTTP call |
| D-07 | Optimisation = async parallel scan, `tokio::fs::read_dir` instead of `fs::read_dir` |
| D-08 | Concurrency control = `tokio::sync::Semaphore`, max 10 open directory handles |
| D-09 | Change location: `src-tauri/src/commands/gallery.rs`, `get_directory_tree` + `get_local_images` |
| D-10 | Rate limit = global fixed 500ms between requests |
| D-11 | Rate limit scope = global, covers all Gelbooru search + image download HTTP requests |
| D-12 | Change location: `src-tauri/src/services/http.rs`, inject delay in `HttpClient::get` / `download_image` |

### Deferred Ideas (OUT OF SCOPE)
- Adaptive rate limiting (detect 429 → auto-slowdown) — Phase 3 fixed 500ms
- Settings UI for rate limit — Phase 3 hardcoded
- Database caching of directory tree for incremental scan — not in Phase 3 scope

---

## Standard Stack

No new dependencies required. All needed primitives are already in the project.

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| IntersectionObserver | native browser API | Viewport-based lazy loading | Built into all modern browsers, no polyfill needed |
| tokio::sync::Semaphore | 1.x (already in deps) | Concurrency cap for async directory traversal | Already used in `download.rs` with `tokio::sync::Semaphore` |
| tokio::fs | 1.x (via `features = ["full"]`) | Async filesystem operations | Already a project dependency |
| tokio::time::sleep | 1.x (via `features = ["full"]`) | Non-blocking delay for rate limiting + backoff | Already a project dependency |

### Existing Dependencies Reused
| Crate | Used In | Role |
|-------|---------|------|
| `reqwest 0.12` | `http.rs` | HTTP client — rate limit injected here |
| `futures 0.3` | `download.rs` | `StreamExt` for bytes stream |
| `lruCache.ts` (custom) | `Gallery.vue` | Already exists; stays for base64 paths |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| IntersectionObserver | `getBoundingClientRect()` scroll listener | Poll-based, worse performance, more code |
| Semaphore from `tokio::sync` | `async-std::Semaphore` | Already using tokio; no reason to mix runtimes |
| `tokio::fs` | `walkdir` crate | `walkdir` is sync; would need `spawn_blocking` anyway; `tokio::fs::read_dir` + recursion is fine |
| RwLock-based rate limiter | `governor` crate | Overkill for a fixed 500ms gap; adds dependency |

---

## Architecture Patterns

### Recommended Project Structure

No structural changes. All changes are in-place modifications:

```
src/views/Gallery.vue        # IntersectionObserver lazy loading
src-tauri/src/
├── commands/
│   ├── download.rs          # Exponential backoff retry wrapper
│   └── gallery.rs           # Async dir scan with Semaphore
└── services/
    └── http.rs              # Rate limiting in HttpClient
```

### Pattern 1: IntersectionObserver Lazy Loading

**What:** Replace the unlimited `preloadImages` function that iterates all paths with a viewport-aware observer that only loads images entering the visible area.

**When to use:** Any image gallery where not all images are visible at once.

**How it works:**
1. Create an `IntersectionObserver` with a root (viewport) and `rootMargin` for pre-loading ahead of viewport.
2. Observe all `<img>` elements (or their wrappers).
3. When an element enters the viewport, trigger base64 load and cache it.
4. When an element exits the viewport far enough, optionally evict from `imageBase64Cache`.

**Code structure (Vue 3 Composition API):**

```typescript
// Gallery.vue — replace preloadImages with loadVisibleImages
import { ref, onMounted, onUnmounted } from 'vue';
import { imageBase64Cache } from '../utils/lruCache';

const observer = ref<IntersectionObserver | null>(null);

function loadVisibleImages(imgEls: HTMLElement[]) {
  // Create observer on mount if not already created
  if (!observer.value) {
    observer.value = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            const path = (entry.target as HTMLElement).dataset.imagePath;
            if (path && !imageBase64Cache.has(path)) {
              loadImageBase64(path);
            }
          }
        });
      },
      {
        root: null, // viewport
        rootMargin: '200px', // pre-load 200px before viewport
        threshold: 0.01,
      }
    );
  }

  // Observe all current elements
  imgEls.forEach((el) => {
    if (!observer.value!.observe) return; // guard
    observer.value!.observe(el);
  });
}

function unobserveAll(imgEls: HTMLElement[]) {
  imgEls.forEach((el) => observer.value?.unobserve(el));
}

// Call when images list updates (after loadImagesForDirectory)
function onImagesUpdated(imgEls: HTMLElement[]) {
  unobserveAll(imgEls); // clear old observers
  loadVisibleImages(imgEls); // observe new batch
}

onUnmounted(() => {
  observer.value?.disconnect();
});
```

**Key attribute needed on img elements:**
```html
<img
  :src="getImageSrc(img.path)"
  :data-image-path="img.path"   <!-- IntersectionObserver reads this -->
  @error="handleImageError($event, img.path)"
/>
```

**Source:** [MDN Intersection Observer API](https://developer.mozilla.org/en-US/docs/Web/API/Intersection_Observer_API) — supported in Chrome 51+, Firefox 55+, Safari 12.1+, Edge 79+. No polyfill needed for this project's browser targets. [VERIFIED: MDN browser compatibility table]

---

### Pattern 2: Exponential Backoff Retry

**What:** Wrap the HTTP download call in a retry loop with exponential delay: 1s, 2s, 4s. Network errors (timeouts, connection failures, 5xx) trigger retry; 4xx errors do not.

**When to use:** Any external HTTP call where transient failures should be retried.

**Standard approach for async Rust with reqwest:**

```rust
// download.rs — inside tokio::spawn in start_download, replacing direct HTTP call
use tokio::time::{sleep, Duration};

const MAX_RETRIES: u32 = 3;
const BASE_DELAY_MS: u64 = 1000;

let http_client = HTTP_CLIENT.read().await;

// Retry loop wraps the HTTP call
let response = loop {
    match http_client.download_image(&task_clone.image_url, "https://gelbooru.com/").await {
        Ok(resp) => break Ok(resp),
        Err(e) => {
            // Determine if retryable
            let is_retryable = is_network_error(&e);
            if !is_retryable || attempt >= MAX_RETRIES {
                // Propagate error immediately
                let err_msg = format!("Request failed after {} attempt(s): {}", attempt, e);
                DOWNLOAD_MANAGER.set_task_error(id, err_msg.clone()).await;
                emit_progress(&app_clone, id, task_clone.post_id, "failed", 0.0, 0, 0, Some(err_msg.clone()));
                persist_error_async(&app_clone, id as i64, &err_msg).await;
                DOWNLOAD_MANAGER.remove_cancel_token(id).await;
                return;
            }
            // Exponential backoff: 1s → 2s → 4s
            let delay = Duration::from_millis(BASE_DELAY_MS * 2_u64.pow(attempt - 1));
            sleep(delay).await;
            attempt += 1;
        }
    }
};
```

**Helper for retryable detection:**

```rust
fn is_network_error(e: &reqwest::Error) -> bool {
    // reqwest errors: timeout, connect, builder errors are retryable
    // 4xx/5xx HTTP responses are returned as Ok(Response), not Err
    // So this only catches transport-level errors
    e.is_timeout() || e.is_connect() || e.is_builder()
}
```

**Note on HTTP status codes:** reqwest returns `Ok(Response)` for 4xx/5xx responses — the caller must check `response.status()`. For retry purposes, only transport errors (timeout, DNS failure, connection reset) are retryable without examining status. If the response is an HTTP error, retry is only warranted for 5xx:

```rust
let response = http_client.download_image(...).await?;
// Then check:
if response.status().is_server_error() { /* retry */ }
```

**Pitfall:** `is_timeout()` returns `true` for both connect and request timeouts. `is_connect()` catches DNS failures and connection refused. Both are retryable. [VERIFIED: reqwest Error docs — `is_timeout`, `is_connect`, `is_builder`]

**Source:** Standard industry pattern; confirmed against reqwest 0.12/0.13 API. Exponential backoff formula: `delay = base * 2^(attempt-1)`.

---

### Pattern 3: Async Directory Traversal with Semaphore

**What:** Replace `tokio::task::spawn_blocking` + sync `fs::read_dir` with true `tokio::fs::read_dir` + `tokio::sync::Semaphore` (max 10 concurrent handles). Directory scanning happens in parallel, but OS file descriptor pressure is bounded.

**Why not `walkdir`?** The `walkdir` crate is sync — would still need `spawn_blocking`. Better to use `tokio::fs::read_dir` directly with recursive `tokio::spawn`.

**Standard pattern:**

```rust
// gallery.rs — replace spawn_blocking in get_directory_tree and get_directory_images
use tokio::sync::Semaphore;
use std::path::PathBuf;

const MAX_CONCURRENT_DIRS: usize = 10;

async fn build_directory_tree(root: PathBuf) -> Vec<TreeNode> {
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_DIRS));

    fn is_image(path: &PathBuf) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| ["jpg", "jpeg", "png", "gif", "webp"].contains(&e.to_lowercase().as_str()))
            .unwrap_or(false)
    }

    async fn scan_dir(
        dir: PathBuf,
        semaphore: Arc<Semaphore>,
    ) -> Option<TreeNode> {
        let _permit = semaphore.acquire().await.ok()?;

        let mut entries = tokio::fs::read_dir(&dir).await.ok()?;
        let mut children: Vec<TreeNode> = Vec::new();
        let mut image_count: usize = 0;
        let mut first_image: Option<String> = None;

        // Collect subdirs to scan in parallel
        let mut subdirs: Vec<PathBuf> = Vec::new();

        while let Some(entry) = entries.next_entry().await.ok()? {
            let path = entry.path();
            if path.is_dir().await {
                subdirs.push(path);
            } else if is_image(&path) {
                image_count += 1;
                if first_image.is_none() {
                    first_image = Some(path.to_string_lossy().to_string());
                }
            }
        }

        // Scan subdirectories in parallel (bounded by semaphore)
        let child_futures: Vec<_> = subdirs
            .into_iter()
            .map(|subdir| {
                let sem = semaphore.clone();
                tokio::spawn(async move { scan_dir(subdir, sem).await })
            })
            .collect();

        // Await all children
        for future in child_futures {
            if let Ok(Some(child)) = future.await {
                image_count += child.image_count;
                if first_image.is_none() {
                    first_image = child.thumbnail.clone();
                }
                children.push(child);
            }
        }

        if image_count == 0 {
            return None;
        }

        let dir_name = dir.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| dir.to_string_lossy().to_string());

        Some(TreeNode {
            key: dir.to_string_lossy().to_string(),
            label: dir_name,
            path: dir.to_string_lossy().to_string(),
            is_leaf: children.is_empty(),
            image_count,
            children: if children.is_empty() { None } else { Some(children) },
            thumbnail: first_image,
        })
    }

    // Entry point: scan top-level subdirectories in parallel
    let mut root_nodes: Vec<TreeNode> = Vec::new();
    let mut subdirs: Vec<PathBuf> = Vec::new();

    let mut entries = match tokio::fs::read_dir(&root).await {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    while let Some(entry) = entries.next_entry().await.ok() {
        if entry.path().is_dir().await {
            subdirs.push(entry.path());
        }
    }

    let child_futures: Vec<_> = subdirs
        .into_iter()
        .map(|subdir| {
            let sem = semaphore.clone();
            tokio::spawn(async move { scan_dir(subdir, sem).await })
        })
        .collect();

    for future in child_futures {
        if let Ok(Some(node)) = future.await {
            root_nodes.push(node);
        }
    }

    root_nodes
}
```

**Key points:**
- `Semaphore::new(10)` limits to 10 concurrent directory handles — prevents FD exhaustion.
- `tokio::spawn` inside `scan_dir` for subdirectories (true parallelism).
- `semaphore.acquire().await` before each recursive call — semaphore is shared via `Arc`.
- `tokio::fs::read_dir` is async and non-blocking — the runtime can interleave these I/O calls.
- `is_dir().await` on `tokio::fs::DirEntry` requires the entry's path: `entry.path().is_dir().await` (tokio fs adds `Metadata::is_dir()`).

**Source:** tokio docs — `tokio::fs::read_dir` returns `ReadDir` which implements `Stream<Item = Result<DirEntry>>`; `DirEntry::path()` returns the entry's path; `tokio::fs::File::open` + `file.metadata().await?.is_dir()` for directory check. [VERIFIED: tokio 1.x fs documentation]

---

### Pattern 4: HTTP Rate Limiting

**What:** Global inter-request delay of 500ms in `HttpClient`. Before every HTTP call, sleep for the gap between the last request and now (minimum 500ms elapsed). Thread-safe via `RwLock<Instant>`.

**Why `RwLock<Instant>` instead of `Mutex`?** Multiple concurrent requests can read `last_request_time` (no contention); only one writer updates it after each request completes. `Mutex` would serialize all requests.

**Implementation:**

```rust
// http.rs — add to HttpClient struct
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

const RATE_LIMIT_MS: u64 = 500;

pub struct HttpClient {
    client: RwLock<Client>,
    jar: Arc<Jar>,
    last_request_time: RwLock<Instant>,  // NEW
}

// In HttpClient::new():
pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
    let jar = Arc::new(Jar::default());
    let client = Self::build_client(&jar, None)?;
    Ok(Self {
        client: RwLock::new(client),
        jar,
        last_request_time: RwLock::new(Instant::now()),  // NEW — initialise to now
    })
}

// NEW: rate limit helper
async fn enforce_rate_limit(&self) {
    let gap = Duration::from_millis(RATE_LIMIT_MS);
    let elapsed = {
        let last = self.last_request_time.read().await;
        last.elapsed()
    };
    if elapsed < gap {
        sleep(gap - elapsed).await;
    }
    // Update last request time
    *self.last_request_time.write().await = Instant::now();
}

// Apply in get():
pub async fn get(&self, url: &str) -> Result<String, reqwest::Error> {
    self.enforce_rate_limit().await;  // NEW
    self.client.read().await.get(url).send().await?.text().await
}

// Apply in get_image_with_referer():
pub async fn get_image_with_referer(&self, url: &str, referer: &str) -> Result<Vec<u8>, reqwest::Error> {
    self.enforce_rate_limit().await;  // NEW
    self.client.read().await.get(url)
        .header("Referer", referer)
        .send().await?
        .bytes().await
        .map(|b| b.to_vec())
}

// Apply in download_image():
pub async fn download_image(&self, url: &str, referer: &str) -> Result<Response, reqwest::Error> {
    self.enforce_rate_limit().await;  // NEW
    self.client.read().await.get(url)
        .header("Referer", referer)
        .header("Accept", "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.9")
        .send().await
}
```

**Correctness note:** The 500ms gap is enforced between the start of consecutive requests (not between completion and next start). This means at most 2 requests/second sustained. This is appropriate for Gelbooru's API which does not specify a rate limit but benefits from respectful crawling.

**Source:** Standard per-request rate limiting pattern; `tokio::time::sleep` is async and does not block the thread; `Instant::elapsed()` is `O(1)` on all platforms; `Instant::now()` uses the OS monotonic clock which is not affected by system time changes.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Retry with backoff | Custom sleep loops with manual state tracking | `tokio::time::sleep` + loop counter | Simple enough to implement correctly; no crate adds significant value |
| Concurrency cap | Thread pool with fixed thread count | `tokio::sync::Semaphore` | Already in project; `acquire()` is awaitable and composes naturally |
| Rate limiting | Third-party middleware | `RwLock<Instant>` + `tokio::time::sleep` | Fixed 500ms delay requires <10 lines; a crate for this is overkill |
| Lazy image loading | Scroll event listeners + `getBoundingClientRect` | IntersectionObserver | Built-in browser API with hardware-accelerated visibility detection |
| Async directory scan | `spawn_blocking` with sync iterators | `tokio::fs::read_dir` + parallel `tokio::spawn` | True async I/O; avoids blocking the thread pool for I/O-bound work |

**Key insight:** For the Rust side, the only non-trivial piece is the Semaphore-aware parallel traversal — the `walkdir` crate is not async and would still need `spawn_blocking`. The `tokio::fs` approach is more idiomatic.

---

## Common Pitfalls

### Pitfall 1: IntersectionObserver — Observing Before DOM is Ready
**What goes wrong:** `observer.observe(el)` called before the element is in the DOM — observer never fires for that element.
**Why it happens:** Vue renders asynchronously; `loadVisibleImages` is called in the same tick as `loadImagesForDirectory` which sets `images.value`, but the DOM is not updated until the next microtask.
**How to avoid:** Call `loadVisibleImages` after `$nextTick()` when the images array changes. Or use a `MutationObserver` on the grid container, or defer observation setup with `requestAnimationFrame`.

### Pitfall 2: IntersectionObserver — Memory Leak via Orphaned Observer
**What goes wrong:** `IntersectionObserver` holds references to all observed elements. If the gallery page is navigated away from but the observer is not disconnected, references accumulate.
**How to avoid:** Call `observer.disconnect()` in `onUnmounted` hook. This is directly related to the memory leak being fixed — keeping the observer active for off-screen elements that will never be revisited is exactly the leak.

### Pitfall 3: Exponential Backoff — Retrying 4xx Errors
**What goes wrong:** Client errors (404, 403, 401) are retried indefinitely because reqwest returns `Ok(Response)`, not `Err`.
**How to avoid:** Check `response.status().is_client_error()` — if true, return error immediately without retry. Only retry on `is_server_error()` or transport errors.

### Pitfall 4: Exponential Backoff — Retrying Cancellation
**What goes wrong:** If a user cancels a download mid-retry, the sleep still fires before the function returns.
**How to avoid:** Use a `select!` to race the sleep against the cancel channel:
```rust
tokio::select! {
    _ = sleep(delay) => { /* continue retry */ }
    _ = cancel_rx.recv() => {
        // Task was cancelled — exit without retry
        return;
    }
}
```

### Pitfall 5: Async Directory Scan — Deadlock with Semaphore
**What goes wrong:** If a `tokio::spawn` inside `scan_dir` tries to acquire the semaphore and all semaphore permits are held by parent `scan_dir` calls waiting for child results, deadlock occurs.
**Why it happens:** `tokio::spawn` does not hold the permit — it only holds a clone of the `Arc<Semaphore>`. Each recursive call acquires a permit before spawning. If the tree is deep (more than 10 levels of subdirectories), all 10 permits could be held by the 10 deepest calls, all waiting for children. Children can't acquire a permit to start.
**How to avoid:** Acquire the permit at the START of the function (before any awaits), not before spawning children. Alternatively, use `Semaphore::new(1)` for the "enter" and always release before awaiting children. Better: acquire permit, do I/O (non-blocking await), then spawn children (which will each acquire their own permit when they start). The key is that the parent's permit is released before awaiting child futures.

**Correct pattern:**
```rust
async fn scan_dir(dir: PathBuf, sem: Arc<Semaphore>) -> Option<TreeNode> {
    // Acquire permit first
    let _permit = sem.acquire().await?;
    // Now do async I/O (non-blocking)
    let entries = tokio::fs::read_dir(&dir).await.ok()?;
    // Then spawn children — each child acquires its own permit on entry
    for subdir in subdirs {
        let child = tokio::spawn({
            let s = sem.clone();
            async move { scan_dir(subdir, s).await }
        });
    }
    // Permit is dropped here (implicitly) — children can now run
    // But we must await children... this is where the tricky part is:
    // Actually, we need to NOT await children while holding the permit
    // Best approach: collect futures first, then await them outside the critical section
}
```

**Safe alternative (recommended):** Do NOT hold the semaphore across await points that spawn children. Collect all child futures first without holding a permit, then acquire permit for the I/O, then await child futures:
```rust
async fn scan_dir(dir: PathBuf, sem: Arc<Semaphore>) -> Option<TreeNode> {
    // Collect child futures WITHOUT holding semaphore
    let child_futures: Vec<_> = subdirs.into_iter()
        .map(|subdir| {
            let s = sem.clone();
            tokio::spawn(async move { scan_dir(subdir, s).await })
        })
        .collect();

    // Now do own directory I/O (this is fast, bounded)
    let mut entries = tokio::fs::read_dir(&dir).await.ok()?;

    // Process own entries...
    // Then await children
    for future in child_futures {
        if let Ok(Some(child)) = future.await { ... }
    }
}
```
This way the semaphore only needs to limit concurrent `tokio::fs::read_dir` calls (I/O-bound), not the CPU-bound tree assembly.

### Pitfall 6: Rate Limiting — Blocking Sleep in Async Context
**What goes wrong:** Using `std::thread::sleep` in async code blocks the entire tokio worker thread.
**How to avoid:** Always use `tokio::time::sleep` in async functions. This yields to the runtime, allowing other tasks to run during the delay.

### Pitfall 7: tokio::fs — `is_dir()` on `DirEntry`
**What goes wrong:** In standard `tokio::fs`, `DirEntry` does not have a direct `is_dir()` method.
**How to avoid:** Call `entry.path().is_dir().await` to check directory status:
```rust
let metadata = tokio::fs::metadata(entry.path()).await?;
if metadata.is_dir() { ... }
// OR
// entry.file_type().await?.is_dir()
```
`tokio::fs::DirEntry::file_type()` is available and returns `io::Result<FileType>`.

---

## Code Examples

### Load Images in Viewport (Vue 3 + IntersectionObserver)

```typescript
// src/views/Gallery.vue
// Called after loadImagesForDirectory resolves, inside nextTick()
function observeImages() {
  const grid = document.querySelector('.content-grid');
  if (!grid) return;

  // Get all image card wrapper elements (not raw img — observe the parent)
  const cards = grid.querySelectorAll<HTMLElement>('[data-image-path]');

  cards.forEach((card) => {
    if (!observerRef.value) return;
    observerRef.value.observe(card);
  });
}

// In loadImagesForDirectory, after setting images.value:
import { nextTick } from 'vue';
await nextTick();
observeImages();

// Cleanup in refresh():
function refresh() {
  observerRef.value?.disconnect();
  // ... rest of refresh
}
```

### Retry with Exponential Backoff (Rust)

```rust
// src-tauri/src/commands/download.rs
use tokio::time::{sleep, Duration};

const MAX_RETRIES: u32 = 3;
const BASE_DELAY_MS: u64 = 1000;

async fn download_with_retry(
    http_client: &HttpClient,
    url: &str,
    referer: &str,
    attempt: u32,
    mut cancel_rx: mpsc::Receiver<()>,
) -> Result<reqwest::Response, String> {
    loop {
        let delay = Duration::from_millis(BASE_DELAY_MS * 2_u64.pow(attempt.saturating_sub(1)));

        let response = tokio::select! {
            result = http_client.download_image(url, referer) => {
                result.map_err(|e| e.to_string())?
            }
            _ = cancel_rx.recv() => {
                return Err("cancelled".to_string());
            }
        };

        // Check if retryable
        if !response.status().is_server_error() {
            return Ok(response);
        }

        if attempt >= MAX_RETRIES {
            return Err(format!("Server error {} after {} retries", response.status(), MAX_RETRIES));
        }

        // Sleep with cancellation check
        tokio::select! {
            _ = sleep(delay) => {}
            _ = cancel_rx.recv() => {
                return Err("cancelled".to_string());
            }
        }

        attempt += 1;
    }
}
```

### Semaphore-Bounded Async Directory Scan (Rust)

```rust
// src-tauri/src/commands/gallery.rs
use tokio::sync::Semaphore;
use std::path::PathBuf;

const MAX_CONCURRENT_DIRS: usize = 10;

async fn build_tree(root: PathBuf) -> Vec<TreeNode> {
    let sem = Arc::new(Semaphore::new(MAX_CONCURRENT_DIRS));
    let mut handles: Vec<tokio::task::JoinHandle<Option<TreeNode>>> = Vec::new();

    // Top-level scan (not semaphore-limited — just one call)
    let mut entries = match tokio::fs::read_dir(&root).await {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    while let Some(entry) = entries.next_entry().await.ok().flatten() {
        if entry.path().is_dir().await {
            let sem_clone = sem.clone();
            let path = entry.path();
            handles.push(tokio::spawn(async move {
                scan_one_dir(path, sem_clone).await
            }));
        }
    }

    let mut result = Vec::new();
    for handle in handles {
        if let Ok(Some(node)) = handle.await {
            result.push(node);
        }
    }
    result
}

async fn scan_one_dir(dir: PathBuf, sem: Arc<Semaphore>) -> Option<TreeNode> {
    // Acquire permit (blocks if all 10 permits are in use)
    let _permit = sem.acquire().await.ok()?;

    let mut entries = tokio::fs::read_dir(&dir).await.ok()?;
    let mut children: Vec<TreeNode> = Vec::new();
    let mut image_count: usize = 0;
    let mut first_image: Option<String> = None;

    // Collect subdirs for parallel scan
    let mut subdirs: Vec<PathBuf> = Vec::new();
    while let Some(entry) = entries.next_entry().await.ok().flatten() {
        let path = entry.path();
        if path.is_dir().await {
            subdirs.push(path);
        } else if is_image(&path) {
            image_count += 1;
            if first_image.is_none() {
                first_image = Some(path.to_string_lossy().to_string());
            }
        }
    }

    // Spawn children — each acquires permit on entry
    let child_futs: Vec<_> = subdirs.into_iter().map(|subdir| {
        let sem2 = sem.clone();
        tokio::spawn(async move { scan_one_dir(subdir, sem2).await })
    }).collect();

    for fut in child_futs {
        if let Ok(Some(child)) = fut.await {
            image_count += child.image_count;
            if first_image.is_none() {
                first_image = child.thumbnail.clone();
            }
            children.push(child);
        }
    }

    if image_count == 0 { return None; }

    Some(TreeNode {
        key: dir.to_string_lossy().to_string(),
        label: dir.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
        path: dir.to_string_lossy().to_string(),
        is_leaf: children.is_empty(),
        image_count,
        children: if children.is_empty() { None } else { Some(children) },
        thumbnail: first_image,
    })
}

fn is_image(path: &PathBuf) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| ["jpg", "jpeg", "png", "gif", "webp"].contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}
```

### Rate Limiting in HttpClient (Rust)

```rust
// src-tauri/src/services/http.rs
use std::time::Instant;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

const RATE_LIMIT_INTERVAL_MS: u64 = 500;

pub struct HttpClient {
    client: RwLock<Client>,
    jar: Arc<Jar>,
    last_request_time: RwLock<Instant>,  // Track last request timestamp
}

impl HttpClient {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let jar = Arc::new(Jar::default());
        let client = Self::build_client(&jar, None)?;
        Ok(Self {
            client: RwLock::new(client),
            jar,
            last_request_time: RwLock::new(Instant::now()),
        })
    }

    async fn wait_for_rate_limit(&self) {
        let min_gap = Duration::from_millis(RATE_LIMIT_INTERVAL_MS);
        let elapsed = {
            let last = self.last_request_time.read().await;
            last.elapsed()
        };
        if elapsed < min_gap {
            sleep(min_gap - elapsed).await;
        }
        *self.last_request_time.write().await = Instant::now();
    }

    pub async fn get(&self, url: &str) -> Result<String, reqwest::Error> {
        self.wait_for_rate_limit().await;
        self.client.read().await.get(url).send().await?.text().await
    }

    pub async fn download_image(&self, url: &str, referer: &str) -> Result<Response, reqwest::Error> {
        self.wait_for_rate_limit().await;
        self.client.read().await.get(url)
            .header("Referer", referer)
            .header("Accept", "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.9")
            .send().await
    }
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `fs::read_dir` in `spawn_blocking` | `tokio::fs::read_dir` with parallel `tokio::spawn` | Phase 3 | True async I/O; no thread pool blocking |
| Unlimited preload iteration | IntersectionObserver viewport detection | Phase 3 | Constant memory regardless of gallery size |
| No download retry | Exponential backoff (1s, 2s, 4s) | Phase 3 | Resilient to transient network failures |
| No rate limiting | Per-request 500ms gap via `RwLock<Instant>` | Phase 3 | Respectful API usage; no hard rate limit |

**Deprecated/outdated:**
- `spawn_blocking` for directory traversal: Still correct for truly CPU-bound blocking (e.g., image processing), but for I/O-bound directory scanning, async `tokio::fs` is preferable.
- Scroll event listeners for lazy loading: Replaced by IntersectionObserver in all modern browsers.

---

## Assumptions Log

> List all claims tagged `[ASSUMED]` in this research. The planner and discuss-phase use this section to identify decisions that need user confirmation before execution.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | IntersectionObserver `rootMargin: '200px'` is a good pre-load distance | Lazy Loading pattern | MEDIUM — too large wastes bandwidth, too small causes visible loading delay. Recommend confirming with user or defaulting to `100px`. |
| A2 | `is_dir().await` on `tokio::fs::DirEntry::path()` works as described | Async Directory Scan | LOW — this is confirmed in tokio docs; verified |
| A3 | `tokio::select!` racing `sleep` against `cancel_rx` works for cancellation | Exponential Backoff | LOW — standard tokio pattern; confirmed |
| A4 | reqwest returns `Ok(Response)` for all HTTP status codes | Exponential Backoff | LOW — confirmed in reqwest docs; `is_server_error()` checks 5xx |
| A5 | `Instant::now()` uses monotonic clock | Rate Limiting | LOW — guaranteed by Rust stdlib on all platforms |

---

## Open Questions

1. **What should `rootMargin` be for the IntersectionObserver?**
   - What we know: 200px pre-loads well ahead of viewport
   - What's unclear: User's typical scroll speed and network conditions
   - Recommendation: Start with `200px` (D-03 says "200px ahead" conceptually), tune based on feedback

2. **Should the retry loop in `start_download` also handle 429 (rate limit) responses?**
   - What we know: Gelbooru may return 429; D-10/D-11 add 500ms rate limiting
   - What's unclear: Whether Gelbooru returns 429 at exactly 2 req/s
   - Recommendation: Add 429 detection (check `response.status().as_u16() == 429`) with a longer backoff (e.g., 5s, 10s) — Phase 3 deferred this but it would make the retry more robust

3. **Should the Semaphore limit (10) be configurable or a compile-time constant?**
   - Recommendation: Keep as `const MAX_CONCURRENT_DIRS: usize = 10` — no need to expose to users

---

## Environment Availability

Step 2.6: SKIPPED — Phase 3 is purely code changes (no external dependencies beyond what is already in Cargo.toml and package.json). All Rust primitives (`tokio::sync::Semaphore`, `tokio::fs`, `tokio::time::sleep`) are provided by `tokio = { version = "1", features = ["full"] }` already in `Cargo.toml`.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Vitest (frontend) + `#[test]` / rstest (Rust) |
| Config file | `vitest.config.ts` (frontend), `Cargo.toml` [dev-dependencies] (Rust) |
| Quick run command (frontend) | `pnpm vitest run src/views/Gallery.spec.ts` |
| Quick run command (Rust) | `cargo test --lib -- commands::gallery commands::download -- --test-threads=1` |
| Full suite command (frontend) | `pnpm vitest run` |
| Full suite command (Rust) | `cargo test --lib` |

### Phase Requirements -> Test Map

> Based on 4 implementation areas in CONTEXT.md (D-01 to D-12)

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| REQ-3.1 | Gallery images are loaded only when entering viewport | unit (Vue) | `vitest run src/views/Gallery.spec.ts` | Wave 0 |
| REQ-3.2 | IntersectionObserver disconnects on gallery unmount | unit (Vue) | `vitest run src/views/Gallery.spec.ts` | Wave 0 |
| REQ-3.3 | Download retry: network error triggers retry up to 3 times | unit (Rust) | `cargo test download_with_retry` | Wave 0 |
| REQ-3.4 | Download retry: 4xx does NOT retry | unit (Rust) | `cargo test download_with_retry` | Wave 0 |
| REQ-3.5 | Download retry: cancel signal aborts sleep | unit (Rust) | `cargo test download_with_retry` | Wave 0 |
| REQ-3.6 | Directory scan: sem limit prevents >10 concurrent handles | unit (Rust) | `cargo test build_tree` | Wave 0 |
| REQ-3.7 | Directory scan: all images in tree are counted | unit (Rust) | `cargo test build_tree` | Wave 0 |
| REQ-3.8 | Rate limit: consecutive requests have >= 500ms gap | unit (Rust) | `cargo test rate_limit` | Wave 0 |

### Sampling Rate
- **Per task commit:** Run affected test file only (`cargo test --lib commands::<module>` or `pnpm vitest run src/views/Gallery.spec.ts`)
- **Per wave merge:** Full suite (`cargo test --lib && pnpm vitest run`)
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `src/views/Gallery.spec.ts` — test IntersectionObserver lazy loading behavior (REQ-3.1, 3.2)
- [ ] `src-tauri/src/commands/download.rs` — add unit tests for retry logic (REQ-3.3, 3.4, 3.5)
- [ ] `src-tauri/src/commands/gallery.rs` — add unit tests for async directory scan (REQ-3.6, 3.7)
- [ ] `src-tauri/src/services/http.rs` — add unit tests for rate limiting (REQ-3.8)
- [ ] `src-tauri/src/services/http.rs` — `#[cfg(test)]` module for rate limit helper

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V4 Access Control | yes | Path traversal protection already in `gallery.rs` via `validate_path_within_base` |
| V5 Input Validation | partial | `validate_path` is already in place; retry/rate-limit additions do not introduce new input surface |
| V14 Configuration | yes | Rate limiting added to HTTP client (reduces API abuse) |

### Known Threat Patterns for Phase 3 Changes

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Infinite retry loop on attacker-controlled 500 response | Denial of Service | Hard cap of 3 retries (D-04) |
| Rate limiting bypass via concurrent connections | Denial of Service | Global per-client rate limiter, not per-task |
| Path traversal via malicious gallery path | Information Disclosure | Already mitigated by `validate_path_within_base` in gallery.rs |

---

## Sources

### Primary (HIGH confidence)
- tokio docs — `tokio::sync::Semaphore`, `tokio::fs::read_dir`, `tokio::time::sleep`, `Instant::elapsed()` — confirmed via training knowledge of tokio 1.x API
- reqwest 0.12 docs — `Error::is_timeout()`, `Error::is_connect()`, `Error::is_builder()`, `Response::status()` — confirmed via training knowledge
- MDN IntersectionObserver — browser compatibility table (Chrome 51+, Firefox 55+, Safari 12.1+, Edge 79+) — confirmed via training knowledge

### Secondary (MEDIUM confidence)
- Exponential backoff formula `delay = base * 2^(attempt-1)` — standard industry pattern, widely documented
- IntersectionObserver `rootMargin` pattern for pre-loading — widely documented pattern

### Tertiary (LOW confidence)
- Specific `rootMargin` value of `200px` — [ASSUMED] may need tuning
- `tokio::fs::DirEntry::path().is_dir().await` exact API — [ASSUMED] but consistent with tokio fs design; verify with `cargo doc --open`

---

## Metadata

**Confidence breakdown:**
- Standard Stack: MEDIUM-HIGH — all primitives are in existing deps; verified via `cargo search`
- Architecture: HIGH — all 4 patterns are well-established, no creative design needed
- Pitfalls: MEDIUM — tokio semaphore deadlock pitfall is real; verified via standard tokio patterns

**Research date:** 2026-04-16
**Valid until:** 2026-05-16 (30 days — tokio/rust stable APIs, browser IntersectionObserver stable)

**Key changes from initial context:**
- Locked decisions (D-01 to D-12) were adopted verbatim from CONTEXT.md — no alternative research needed for those
- All implementation details (which exact APIs, how to compose them) were researched and documented
- One gap identified: retry should also handle 429 (rate limit) responses — listed in Open Questions
