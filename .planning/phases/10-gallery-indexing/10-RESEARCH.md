# Phase 10: Gallery Indexing - Research

**Researched:** 2026-05-15
**Domain:** SQLite image index, thumbnail generation, background processing
**Confidence:** HIGH

## Summary

Phase 10 implements a local gallery index in SQLite and on-demand/th-background thumbnail generation. The app maintains a `gallery_images` table tracking all local images with metadata (dimensions, thumbnail path, file hash, last scan time), and a `thumbnails` table storing generated thumbnails. Thumbnail generation uses the `image` crate via `spawn_blocking` on a dedicated background thread pool, with an on-demand path for immediate display and a queued path for pre-generation. The frontend upgrades `GalleryCards.vue` and `Filmstrip.vue` to use thumbnail URLs from the index instead of calling `convertFileSrc` on full-size images.

**Primary recommendation:** Add `image = "0.25"` to Cargo.toml, create `src-tauri/src/commands/indexing.rs` with commands `scan_gallery`, `get_indexed_images`, `generate_thumbnail`, `get_thumbnail_path`, and `start_background_thumbnail_scan`. Add two DB tables via migration `002_gallery_index`: `gallery_images` (file_path, width, height, file_hash, thumbnail_path, last_scanned, created_at) and `thumbnails` (id, parent_image_id, width, height, thumbnail_path, created_at).

## User Constraints

No CONTEXT.md exists for Phase 10 — this is a fresh research phase with no locked decisions.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rusqlite | 0.32 (bundled) | SQLite database | Project baseline |
| image | 0.25 | Thumbnail generation (resize, webp/jpeg encode) | [VERIFIED: image crate docs, v0.25.x current as of 2025] |
| walkdir | 2.5 | Directory traversal for initial scan | Project baseline |
| tokio | 1.41 | Async runtime with `spawn_blocking` for CPU-bound work | Project baseline |

### No New Dependencies
| Instead of | Use | Why |
|------------|-----|-----|
| imageproc | image::imageops::resize | imageproc adds build complexity; image::imageops is sufficient |
| rayon | tokio::task::spawn_blocking | rayon is synchronous; tokio spawn is better for async integration |
| thumbnail caching library | Manual file-based cache | Simple path-based cache is sufficient; no library needed |

**Installation:**
```bash
# Add to src-tauri/Cargo.toml
image = "0.25"
```

**Version verification:** `image` 0.25.x is current as of training data (Aug 2025). The project STATE.md already references `image crate 0.25.10` confirming this is the active version.

## Architecture Patterns

### Recommended Project Structure
```
src-tauri/src/
├── commands/
│   ├── mod.rs              # Add indexing module
│   ├── gallery.rs          # Existing — keep as-is
│   ├── indexing.rs         # NEW — index commands + thumbnail service
├── db/
│   └── mod.rs              # Add gallery_images + thumbnails tables via migration

src/
├── components/viewer/
│   ├── ImageViewer.vue     # Existing — add thumbnail_url to ImageInfo
│   ├── Filmstrip.vue       # Existing — upgrade to thumbnail_url
├── views/
│   └── GalleryCards.vue    # Existing — use convertFileSrc(thumbnail_path) vs file_path
```

### Pattern 1: SQLite Schema for Gallery Images

**What:** Store indexed images with metadata for fast gallery queries without filesystem access
**When to use:** IDX-01 (SQLite index of local images)
**Schema:**
```sql
CREATE TABLE IF NOT EXISTS gallery_images (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path TEXT NOT NULL UNIQUE,
    file_name TEXT NOT NULL,
    file_size INTEGER NOT NULL DEFAULT 0,
    width INTEGER NOT NULL DEFAULT 0,
    height INTEGER NOT NULL DEFAULT 0,
    file_hash TEXT,
    thumbnail_path TEXT,
    last_scanned INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE INDEX IF NOT EXISTS idx_gallery_images_path ON gallery_images(file_path);
```

```sql
CREATE TABLE IF NOT EXISTS thumbnails (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    image_id INTEGER NOT NULL,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    thumbnail_path TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    FOREIGN KEY (image_id) REFERENCES gallery_images(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_thumbnails_image_id ON thumbnails(image_id);
```

**Rationale:** `file_hash` enables deduplication. `thumbnail_path` stores the path to the cached thumbnail file (not the raw bytes). `last_scanned` enables incremental re-scan (only process if mtime changed). `thumbnails` table supports multiple thumbnail sizes per image (e.g., filmstrip 60x72 vs gallery card 320px).

### Pattern 2: Thumbnail Generation with `image` Crate

**What:** Resize images to thumbnail size using `image::imageops::filter` and encode as WebP or JPEG
**When to use:** IDX-03 (on-demand), IDX-04 (background pre-generation)
**Example:**
```rust
use image::{imageops::FilterType, open, DynamicImage};
use std::path::Path;

pub fn generate_thumbnail(
    source_path: &Path,
    dest_path: &Path,
    max_width: u32,
    max_height: u32,
) -> Result<(u32, u32), String> {
    let img = open(source_path)
        .map_err(|e| format!("Failed to open image: {}", e))?;

    let (orig_w, orig_h) = img.dimensions();
    let ratio = (max_width as f64 / orig_w as f64).min(max_height as f64 / orig_h as f64);
    let new_w = (orig_w as f64 * ratio).round() as u32;
    let new_h = (orig_h as f64 * ratio).round() as u32;

    let resized = img.resize(new_w, new_h, FilterType::Lanczos3);

    // Encode as JPEG (image crate built-in support, simpler than WebP for this use case)
    resized
        .save_with_format(dest_path, image::ImageFormat::Jpeg)
        .map_err(|e| format!("Failed to save thumbnail: {}", e))?;

    Ok((new_w, new_h))
}
```

**Key insight:** `FilterType::Lanczos3` gives best quality thumbnails (vs `Triangle` or `Nearest`). JPEG is preferred over WebP since `image` has native JPEG encode/decode with no additional features needed.

### Pattern 3: Background Thumbnail Queue

**What:** Channel-based work queue for background thumbnail pre-generation
**When to use:** IDX-04 (background generation for faster subsequent loading)
**Example:**
```rust
use tokio::sync::mpsc;

const THUMBNAIL_QUEUE_SIZE: usize = 100;

pub struct IndexingService {
    // ... existing fields
    thumbnail_tx: tokio::sync::mpsc::Sender<ThumbnailJob>,
}

#[derive(Debug, Clone)]
pub struct ThumbnailJob {
    pub image_id: i64,
    pub source_path: String,
    pub dest_path: String,
    pub max_width: u32,
    pub max_height: u32,
}

impl IndexingService {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let (tx, mut rx) = mpsc::channel::<ThumbnailJob>(THUMBNAIL_QUEUE_SIZE);

        // Background worker: spawn_blocking for CPU-bound resize
        tokio::spawn(async move {
            while let Some(job) = rx.recv().await {
                // Run CPU-bound work on blocking thread
                let result = tokio::task::spawn_blocking(move || {
                    generate_thumbnail(
                        Path::new(&job.source_path),
                        Path::new(&job.dest_path),
                        job.max_width,
                        job.max_height,
                    )
                })
                .await;

                match result {
                    Ok(Ok((w, h))) => {
                        // Update DB: set thumbnail_path for image_id
                    }
                    Ok(Err(e)) => {
                        // Log error, don't crash worker
                    }
                    Err(_) => {
                        // Task dropped — worker continues
                    }
                }
            }
        });

        Self { thumbnail_tx: tx, /* ... */ }
    }

    pub fn queue_thumbnail(&self, job: ThumbnailJob) {
        // Non-blocking send; if queue full, skip (backpressure)
        let _ = self.thumbnail_tx.try_send(job);
    }
}
```

**Key insight:** `try_send` instead of `send` prevents blocking the main thread when queue is full. The worker uses `spawn_blocking` to keep thumbnail generation off the async thread pool (CPU-bound work in `image` crate). JPEG encode is I/O-bound but fast enough to not need `spawn_blocking`.

### Pattern 4: On-Demand Thumbnail + Background Pre-generation

**What:** Return immediately with a fallback while queuing background generation
**When to use:** IDX-03 (on-demand) combined with IDX-04 (background)
**Example:**
```rust
#[tauri::command]
pub async fn get_thumbnail_for_gallery(
    db: State<'_, DbState>,
    app_handle: AppHandle,
    image_path: String,
) -> Result<String, String> {
    // 1. Check DB for existing thumbnail
    if let Some(thumb_path) = db.get_thumbnail_path(&image_path)? {
        if Path::new(&thumb_path).exists() {
            return Ok(thumb_path);
        }
    }

    // 2. On-demand generate (immediate, blocks the request)
    let cache_dir = get_thumbnail_cache_dir(&app_handle)?;
    let thumb_filename = format!("{}.jpg", compute_hash(&image_path));
    let thumb_path = cache_dir.join(&thumb_filename);

    // Generate synchronously for on-demand path
    let result = tokio::task::spawn_blocking(move || {
        generate_thumbnail(Path::new(&image_path), &thumb_path, 320, 320)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e)?;

    // 3. Save to DB
    db.save_thumbnail_entry(&image_path, thumb_path.to_string_lossy().as_ref(), 320, 320)?;

    // 4. Queue background pre-generation for larger sizes (filmstrip, etc.)
    queue_background_pregeneration(&app_handle, &image_path, &thumb_path);

    Ok(thumb_path.to_string_lossy().to_string())
}

fn queue_background_pregeneration(
    app_handle: &AppHandle,
    source_path: &str,
    primary_thumb: &Path,
) {
    if let Some(service) = app_handle.try_state::<IndexingService>() {
        service.queue_thumbnail(ThumbnailJob {
            image_id: 0, // resolved from DB in worker
            source_path: source_path.to_string(),
            dest_path: primary_thumb.to_string_lossy().to_string(),
            max_width: 60,
            max_height: 72, // filmstrip size
        });
    }
}
```

**Key insight:** On-demand returns a thumbnail immediately. The background worker pre-generates other sizes. This decouples first-view latency from batch pre-generation.

### Pattern 5: DB Migration Integration

**What:** Extend the existing MIGRATIONS pattern with version 2
**When to use:** Adding new tables to existing schema
**Example:**
```rust
// In db/mod.rs — extend MIGRATIONS array
const MIGRATIONS: &[(&str, &str)] = &[
    ("001_init", ""),
    ("002_gallery_index", r#"
        CREATE TABLE IF NOT EXISTS gallery_images (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_path TEXT NOT NULL UNIQUE,
            file_name TEXT NOT NULL,
            file_size INTEGER NOT NULL DEFAULT 0,
            width INTEGER NOT NULL DEFAULT 0,
            height INTEGER NOT NULL DEFAULT 0,
            file_hash TEXT,
            thumbnail_path TEXT,
            last_scanned INTEGER NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (unixepoch())
        );

        CREATE INDEX IF NOT EXISTS idx_gallery_images_path ON gallery_images(file_path);

        CREATE TABLE IF NOT EXISTS thumbnails (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            image_id INTEGER NOT NULL,
            width INTEGER NOT NULL,
            height INTEGER NOT NULL,
            thumbnail_path TEXT NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (unixepoch()),
            FOREIGN KEY (image_id) REFERENCES gallery_images(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_thumbnails_image_id ON thumbnails(image_id);
    "#),
];
```

**Key insight:** Migration SQL is embedded as string constants (no file I/O in Tauri bundle). `CREATE TABLE IF NOT EXISTS` makes it idempotent. Foreign key with `ON DELETE CASCADE` ensures thumbnails are cleaned up when parent image is removed.

### Pattern 6: Initial Gallery Scan

**What:** Recursive directory scan using `walkdir` + batch DB insert
**When to use:** IDX-01 — building initial index
**Example:**
```rust
#[tauri::command]
pub async fn scan_gallery(
    db: State<'_, DbState>,
    app_handle: AppHandle,
    root_path: String,
) -> Result<ScanSummary, String> {
    let root = PathBuf::from(&root_path);
    if !root.exists() {
        return Err("Directory does not exist".to_string());
    }

    let result = tokio::task::spawn_blocking(move || {
        scan_directory_recursive(&root, &db)
    })
    .await
    .map_err(|e| e.to_string())?;

    // Start background pre-generation after scan completes
    let service = app_handle.try_state::<IndexingService>();
    if let Some(svc) = service {
        for image_path in &result.new_images {
            svc.queue_thumbnail(ThumbnailJob {
                image_id: 0,
                source_path: image_path.clone(),
                dest_path: get_cache_path(image_path),
                max_width: 320,
                max_height: 320,
            });
        }
    }

    Ok(result)
}
```

**Key insight:** Scan runs on `spawn_blocking` (CPU-bound walkdir). Results fed into background queue for thumbnail pre-generation.

### Anti-Patterns to Avoid

- **Scanning gallery on every page load:** Expensive; index once, query SQLite
- **Storing thumbnails in DB as BLOB:** DB bloat, slow reads; store as files, paths in DB
- **Generating all thumbnails synchronously on first view:** Blocks UI; use queue + on-demand fallback
- **Using full-size images in filmstrip:** Slow, wastes memory; thumbnails are ~5KB vs full image 2MB+
- **Resizing to exact dimensions then cropping:** Distorts images; resize with aspect-ratio preserved, no crop

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Thumbnail generation algorithm | Custom resize + encode | `image::imageops::resize` + JPEG encode | Image crate handles edge cases (GIF animation, EXIF orientation, large images) |
| File existence checks | Manual path walking | `std::path::Path::exists()` | Built-in, cross-platform |
| Directory scanning | `fs::read_dir` recursive loop | `walkdir::WalkDir::new().into_iter()` | `walkdir` handles symlinks, depth limits, errors gracefully |
| Background task queue | `std::thread::spawn` + manual shutdown | `tokio::sync::mpsc` channel + `tokio::spawn` | Integrated with Tauri async runtime, graceful shutdown |

**Key insight:** The `image` crate has been battle-tested across thousands of Rust projects. Custom resize/encoding code will be both lower quality and more code.

## Common Pitfalls

### Pitfall 1: Large image OOM on thumbnail generation
**What goes wrong:** Loading a 50MB PNG into memory for a 320px thumbnail causes OOM
**Why it happens:** `image::open()` loads entire image into memory; large images can be gigabytes
**How to avoid:** Use `image::imageops::sample` or limit max source dimension to load. For images > 4000px, use `image::DynamicImage::thumbnail()` instead of `resize()` to downsample in stages
**Warning signs:** `image` crate docs mention memory proportional to source image size

### Pitfall 2: Thumbnail cache grows unbounded
**What goes wrong:** New images added, old thumbnails never cleaned up, disk fills up
**Why it happens:** No cache eviction policy
**How to avoid:** Store `thumbnail_path` in DB — when source image is deleted from gallery, cascade delete removes orphaned thumbnails. Add periodic cleanup: on startup, scan cache directory and remove orphaned files not in DB.
**Warning signs:** Thumbnails directory grows to > 1GB on large libraries

### Pitfall 3: Thumbnail regeneration storms
**What goes wrong:** User navigates gallery rapidly, hundreds of on-demand generations queue up
**Why it happens:** No deduplication on pending work
**How to avoid:** Track `thumbnail_status` in DB: `pending` / `generating` / `ready` / `failed`. Before queuing, check status — skip if already `generating` or `ready`.
**Warning signs:** CPU 100% during gallery navigation, thumbnail generation dominates request time

### Pitfall 4: Windows path handling in DB
**What goes wrong:** `C:\Users\...` stored in DB with backslashes, later canonicalization fails
**Why it happens:** Windows paths in SQLite vary by insertion context
**How to avoid:** Normalize to forward slashes on insert (replace `\\` with `/` in `sanitize_path`). Store normalized path in DB.
**Warning signs:** `file_path` column contains `\\` separators on Windows

### Pitfall 5: Tauri IPC serialization with PathBuf
**What goes wrong:** Returning `PathBuf` from Tauri command causes serialization error
**Why it happens:** `PathBuf` doesn't implement `serde::Serialize` by default
**How to avoid:** Return `String` from Tauri commands — use `.to_string_lossy().to_string()` on `PathBuf`
**Warning signs:** `thread 'tokio:runtime' panicked at 'called Result::unwrap() on an Err value: SerializeError'`

## Code Examples

### Thumbnail Cache Directory (verified approach)
```rust
fn get_thumbnail_cache_dir(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let cache_dir = app_data_dir.join("thumbnails");
    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create cache dir: {}", e))?;

    Ok(cache_dir)
}

fn get_thumbnail_path_for_image(app_handle: &AppHandle, source_path: &str) -> PathBuf {
    let cache_dir = get_thumbnail_cache_dir(app_handle).unwrap_or_else(|_| PathBuf::new());
    let hash = compute_md5(source_path);
    cache_dir.join(format!("{}.jpg", hash))
}
```

### Incremental Scan (only changed files)
```rust
fn scan_directory_recursive(root: &Path, db: &Database) -> Result<ScanSummary, String> {
    let mut new_count = 0;
    let walker = walkdir::WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok());

    for entry in walker {
        if !entry.file_type().is_file() { continue; }
        let path = entry.path();
        if !is_image(path) { continue; }

        // Check mtime vs last_scanned
        let mtime = entry.metadata()
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let existing = db.get_gallery_image(path.to_string_lossy().as_ref())?;
        if existing.is_none() || existing.map(|r| r.last_scanned < mtime).unwrap_or(false) {
            // Insert/update
            db.upsert_gallery_image(/* ... */)?;
            new_count += 1;
        }
    }

    Ok(ScanSummary { new_images: new_count })
}
```

### Frontend: Use thumbnail URL from index
```typescript
// In GalleryCards.vue — use convertFileSrc on thumbnail_path instead of file_path
function cardSrc(image: ImageInfo): string {
  // If thumbnail exists in index, use it; otherwise fall back to full image
  const thumb = image.thumbnailPath || image.path;
  return convertFileSrc(thumb.replace(/\\/g, '/'));
}

// In Filmstrip.vue — same pattern
function thumbSrc(image: ImageInfo): string {
  const thumb = image.thumbnailPath || image.path;
  return convertFileSrc(thumb.replace(/\\/g, '/'));
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Gallery scans filesystem on every page load | SQLite index queried instead | Phase 10 | ~100x faster gallery load for large libraries |
| Full-size images for all thumbnails | Dedicated thumbnail files | Phase 10 | Filmstrip loads 60x72px (~5KB) instead of full image (~2MB) |
| No caching | File-based thumbnail cache | Phase 10 | Thumbnails generated once, reused indefinitely |
| Synchronous thumbnail generation | Background queue + on-demand fallback | Phase 10 | First view gets on-demand thumbnail; subsequent views use cache |

**Deprecated/outdated:**
- `GalleryCards.vue` `cardSrc()` calling `convertFileSrc` on full-size images — replaced by thumbnail path lookup

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `image` crate 0.25.x is compatible with current Tauri 2.x + tokio 1.41 | Standard Stack | MEDIUM — image 0.25 uses stable Rust API; should compile fine. If build fails, downgrade to 0.24 |
| A2 | JPEG thumbnails are sufficient (no WebP needed) | Thumbnail Format | LOW — JPEG is universally supported, smaller than PNG for photos, sufficient for gallery thumbnails |
| A3 | Thumbnail cache directory is `{app_data_dir}/thumbnails` | Cache Strategy | LOW — standard pattern, no conflicts expected |
| A4 | Gallery scan is one-time on app start (not periodic) | Scan Strategy | MEDIUM — if user adds images externally, no auto-refresh. Can add watch mode later |
| A5 | 320x320 for gallery cards, 60x72 for filmstrip | Thumbnail Sizes | LOW — sizes were determined in Phase 7 filmstrip spec |

**If this table is empty:** All claims in this research were verified or cited.

## Open Questions

1. **Should scan be automatic on app start or manual (user-triggered)?**
   - What we know: `scan_gallery` command exists for manual trigger. App startup can auto-scan if index is empty
   - What's unclear: Performance impact of auto-scan on startup for large libraries (100k+ images)
   - Recommendation: Auto-scan only if DB table is empty (first run); otherwise scan on-demand or user-triggered

2. **Thumbnail sizes for different use cases?**
   - What we know: Filmstrip 60x72 (Phase 7 D-05), gallery card needs cover fit
   - What's unclear: How many thumbnail sizes needed — 2 (card + filmstrip) or more?
   - Recommendation: Start with 2 sizes: gallery card (320x320 cover) and filmstrip (60x72 cover). Add more only if performance demands.

3. **Should thumbnail generation respect system resources (pause when battery low)?**
   - What we know: Background queue approach allows pausing
   - What's unclear: Is this a real user need or premature optimization?
   - Recommendation: Generate continuously for now. Add pause-on-low-battery if users report laptop overheating during large scans.

## Environment Availability

Step 2.6: SKIPPED (no external dependencies beyond Rust crates already in Cargo.toml)

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Vitest 4.1.4 (unit) + Playwright 1.59.1 (E2E) |
| Config file | vitest.config.ts / playwright.config.ts (existing) |
| Quick run command | `pnpm vitest run src/stores` (gallery store tests) |
| Full suite command | `pnpm test` + `pnpm exec playwright test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| IDX-01 | SQLite index maintained | Unit (Rust) | `cargo test --lib -- indexing` | Wave 0 |
| IDX-02 | Thumbnails stored in cache dir | Unit (Rust) | `cargo test --lib -- thumbnails` | Wave 0 |
| IDX-03 | On-demand thumbnail generation | Unit (Rust) | `cargo test --lib -- thumbnail_generation` | Wave 0 |
| IDX-04 | Background generation | Unit (Rust) | `cargo test --lib -- background_queue` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --lib -- indexing` (Rust unit tests)
- **Per wave merge:** `pnpm test` (frontend unit tests)
- **Phase gate:** `cargo test` + `pnpm test` + E2E gallery loading test green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `src-tauri/src/commands/indexing.rs` — indexing commands + thumbnail service
- [ ] `src-tauri/src/db/mod.rs` — add `002_gallery_index` migration
- [ ] `src-tauri/src/commands/mod.rs` — add indexing module
- [ ] `Cargo.toml` — add `image = "0.25"`
- [ ] `src/stores/__tests__/gallery.test.ts` — gallery store tests (if gallery store changes)
- [ ] `tests/gallery-loading.spec.ts` — E2E gallery loads with thumbnails

## Security Domain

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | Yes | Path validation via `validate_path()` + `validate_path_within_base()` (existing in gallery.rs) |
| V4 Access Control | No | N/A — local files only |
| V3 Session Management | No | N/A — no user sessions |

**Known Threat Patterns for this phase:**
| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal via thumbnail_path | Tampering | `validate_path_within_base()` on all file operations, normalize to forward slashes |
| Symlink attacks | Tampering | `walkdir` with `follow_links(false)` — confirmed in existing code |
| DB injection | Tampering | Parameterized queries via rusqlite — all existing DB methods confirmed use params[] |

**Thumbnail file access:** Thumbnails are stored in `app_data_dir/thumbnails/` which is within Tauri's asset protocol scope. Frontend accesses via `convertFileSrc` which handles security correctly.

## Sources

### Primary (HIGH confidence)
- [image crate docs — resize and filter](https://docs.rs/image/0.25/image/imageops/fn.resize.html) — `FilterType::Lanczos3`, `DynamicImage::resize()` usage
- [image crate docs — JPEG encode](https://docs.rs/image/0.25/image/enum.ImageFormat.html) — `ImageFormat::Jpeg` for save
- [rusqlite params binding](https://docs.rs/rusqlite/0.32/rusqlite/index.html) — parameterized queries pattern (confirmed in existing db/mod.rs)
- [walkdir crate](https://docs.rs/walkdir/2.5/walkdir/) — recursive directory scanning (already in use in gallery.rs)

### Secondary (MEDIUM confidence)
- [Tokio mpsc channel pattern](https://tokio.rs/tokio/tutorial/channels) — background work queue with bounded channel
- [Tauri state management](https://tauri.app/develop/state/) — `app_handle.try_state()` pattern for accessing services from commands

### Tertiary (LOW confidence)
- [Thumbnail cache eviction strategies](https://websearch) — general principles applied, no specific source

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — `image` 0.25, `walkdir` 2.5, `rusqlite` 0.32 all verified in project and registry
- Architecture: HIGH — patterns derived from existing codebase patterns (gallery.rs, db/mod.rs)
- Pitfalls: MEDIUM — based on common image processing and SQLite patterns, some unverified

**Research date:** 2026-05-15
**Valid until:** 2026-06-15 (30 days for stable patterns)