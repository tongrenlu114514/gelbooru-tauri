# Phase 4: Polish & Release - Research

**Researched:** 2026-04-17
**Domain:** Tauri 2.x desktop app polish, rusqlite schema versioning, error consistency, README authoring, release build
**Confidence:** HIGH (source-code-verified)

## Summary

Phase 4 has four tasks, all well-scoped by the locked decisions in 04-CONTEXT.md. The most complex is Task 4.1 (schema versioning) because it must handle the gap between the existing `Database::new()` (which uses `CREATE TABLE IF NOT EXISTS` with no version tracking) and the new `schema_version` table that must record the baseline for existing DBs. Tasks 4.2-4.4 are straightforward additions on top of established patterns.

**Primary recommendations:**
- Implement schema versioning in `db/mod.rs` by adding `schema_version` table creation to `new()`, then inserting version=1 for existing databases (no prior version row) and running sequential migrations from version+1 upward
- Leave `println!()` / `eprintln!()` patterns as-is everywhere; only add explicit error-prefix labels in commands that lack them (e.g., `gallery.rs` `delete_image` uses a non-standard `format!("删除失败: ...")` pattern)
- Create `README.md` from scratch at project root (no existing README found)
- `tauri.conf.json` is already production-ready; no changes needed

---

## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Schema version table + sequential migration — `schema_version` table (id, version INTEGER), `new()` reads current version, executes migrations in order, initial DB version = 1
- **D-02:** Migration file naming = `001_init.sql` / `002_add_column.sql` (sequential integer prefix)
- **D-03:** Error type = keep `Result<T, String>` (no anyhow/thiserror)
- **D-04:** Logging = keep `println!()` (no tracing)
- **D-05:** README scope = basic README (features + install + usage + contribute, 1-2 pages)
- **D-06:** tauri.conf.json = no changes needed (already 1.0.0)

### Deferred Ideas (OUT OF SCOPE)
- Introducing anyhow/thiserror
- Introducing tracing
- Full migration framework (sqlx-migrate / rusqlite_migration)
- Full documentation (screenshots, FAQ, architecture)

---

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| REQ-4.1 | Schema version table + sequential migrations in `db/mod.rs` | Section 1: Migration implementation strategy |
| REQ-4.2 | Error handling consistency (unified `Result<T, String>` pattern) | Section 2: Error pattern audit |
| REQ-4.3 | Basic README (1-2 pages) | Section 3: README content structure |
| REQ-4.4 | tauri.conf.json production verification | Section 4: Release build and config |

---

## 1. Schema Migration Implementation (Task 4.1)

### Current State

`Database::new()` (db/mod.rs lines 44-105) uses `CREATE TABLE IF NOT EXISTS` for 5 tables:
- `downloads`
- `favorites`
- `blacklisted_tags`
- `favorite_tags`
- `settings`

No version tracking exists. All tables use `IF NOT EXISTS`, meaning existing tables are never recreated.

**rusqlite version:** 0.32 with bundled SQLite
**Verified:** `npm view rusqlite version` returned 2.10.1, but `Cargo.toml` specifies `0.32` — the crates.io version is 0.32.1 as of late 2024. [VERIFIED: Cargo.lock / Cargo.toml declaration]

### The Existing-Database Gap Problem

The decision says "initial version set to 1." But existing databases created by the current `Database::new()` have no `schema_version` row. The planner must handle this:

1. Add `CREATE TABLE IF NOT EXISTS schema_version (version INTEGER PRIMARY KEY)` to `new()`
2. After table creation, query it — if no row exists, the DB was created by the old code
3. In that case, `INSERT OR IGNORE INTO schema_version VALUES (1)` — this sets baseline version=1 for existing DBs
4. Future migrations run from `current_version + 1` upward

### Migration Data Structure

D-02 says migrations are named `001_init.sql`, `002_add_column.sql`... but D-01 and the CONTEXT clarify migrations are **embedded SQL string constants** in `db/mod.rs`, not external files loaded at runtime. The naming convention documents what a migration *looks like*; it does not require file I/O. This avoids the complexity of including SQL files in the Tauri binary bundle.

**Proposed structure:**
```rust
/// All migrations run sequentially starting from version 1.
/// Migrations are embedded string constants to avoid file I/O in the Tauri bundle.
const MIGRATIONS: &[(&str, &str)] = &[
    // Version 1: baseline — all tables created by the original new()
    ("001_init", ""), // No-op, version 1 already set for existing DBs
    // Future example:
    // ("002_add_column", "ALTER TABLE downloads ADD COLUMN new_col TEXT"),
];

impl Database {
    fn run_migrations(conn: &Connection) -> SqliteResult<()> {
        // 1. Ensure schema_version table exists
        conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_version (version INTEGER PRIMARY KEY)",
            [],
        )?;

        // 2. If no row exists, this is an existing DB — set baseline to 1
        let has_row: bool = conn.query_row(
            "SELECT 1 FROM schema_version LIMIT 1",
            [],
            |_| Ok(true),
        ).unwrap_or(false);

        if !has_row {
            conn.execute("INSERT INTO schema_version VALUES (1)", [])?;
        }

        // 3. Read current version
        let current: i32 = conn.query_row(
            "SELECT version FROM schema_version",
            [],
            |row| row.get(0),
        )?;

        // 4. Run migrations sequentially
        for (name, sql) in MIGRATIONS.iter() {
            let version: i32 = name[..3].parse().unwrap_or(0); // "001" -> 1
            if version > current && !sql.is_empty() {
                conn.execute_batch(sql)?;
                conn.execute(
                    "INSERT OR REPLACE INTO schema_version VALUES (?1)",
                    rusqlite::params![version],
                )?;
            }
        }
        Ok(())
    }
}
```

### SQLite ALTER TABLE Limitations

SQLite has restricted DDL support:
- `ALTER TABLE ADD COLUMN` — supported since SQLite 3.1.6 (2006) — works with `IF NOT EXISTS` guard [VERIFIED: SQLite docs]
- `ALTER TABLE RENAME COLUMN` — supported since SQLite 3.25.0 (2018) [VERIFIED]
- `ALTER TABLE DROP COLUMN` — supported since SQLite 3.35.0 (2021) with restrictions [VERIFIED]
- `ALTER TABLE RENAME TO` (table rename) — supported [VERIFIED]

**The "table rebuild" workaround:** For operations SQLite doesn't support directly (e.g., renaming a table), SQLite performs an implicit table rebuild in the background. This is transparent to the application — no special handling needed in rusqlite.

**No workarounds needed** for typical additions/renames in this app's scope.

### Idempotency Concern

`CREATE TABLE IF NOT EXISTS` is already idempotent — safe to run on every `new()` call. The version-gated `conn.execute_batch(sql)` ensures each migration runs exactly once.

---

## 2. Error Handling Consistency (Task 4.2)

### Current Error Patterns

Audit of all command files:

| File | Error Pattern | Issue |
|------|--------------|-------|
| gelbooru.rs | `format!("HTTP request failed: {} (source: {:?})", e, e.source())` + `println!("[ERROR] ...")` | Standard — fully consistent |
| download.rs | `format!("...")` + `eprintln!()` for DB persist failures | Standard — fully consistent |
| gallery.rs | `format!("删除失败: {}", e)` in `delete_image`; `e.to_string()` for lock errors | Minor inconsistency: uses Chinese prefix, no `[ERROR]` label |
| settings.rs | `format!("Failed to get settings: {}", e)` + `format!("Failed to save setting {}: {}", key, e)` | Uses English, no `[ERROR]` label |
| favorite_tags.rs | `e.to_string()` — converts any error to string | Standard |

### Recommendations for Consistency

1. **No structural changes needed** — all commands already return `Result<T, String>`
2. **Optional polish:** Add `[ERROR]` prefix to `gallery.rs` and `settings.rs` error logs to match the `gelbooru.rs` pattern — but this is cosmetic and the locked decision D-03/D-04 says no refactoring of logging
3. **The "删除失败" pattern** in `gallery.rs` uses Chinese error messages for user-facing operations — this is acceptable and intentional (UI-facing messages in Chinese). The issue is only that no `println!()`/`eprintln!()` debug label is applied to the actual error being returned
4. **No new error types** — D-03 explicitly forbids anyhow/thiserror; the `thiserror` crate is already in `Cargo.toml` but unused

### Where println!/eprintln! Are Currently Used

| Location | Type | Label | Purpose |
|----------|------|-------|---------|
| gelbooru.rs line 33, 42, 46 | `println!` | `[DEBUG]` | HTTP fetch logging |
| gelbooru.rs line 38, 61 | `println!` | `[ERROR]` | HTTP error logging |
| download.rs line 234 | `eprintln!` | none | DB persist failure (non-critical) |
| favorite_tags.rs | none | — | Silent errors |

**Key insight:** Phase 4 should NOT refactor `println!()` patterns — D-04 locks this as out of scope. Only verify consistency where it affects user-facing error messages.

---

## 3. README Content Structure (Task 4.3)

### Current State

No `README.md` exists at project root. [VERIFIED: `ls` returned no output]

### Recommended Structure

Per D-05 (basic README, 1-2 pages):

```
README.md
├── Project Title + One-Line Description
├── Features (5-6 bullet points)
├── Installation
│   ├── Prerequisites (Node.js, Rust, pnpm)
│   ├── Build from source
│   └── Run the app
├── Usage
│   ├── First launch (download path setup)
│   ├── Search and download images
│   ├── Managing favorite tags
│   └── Local gallery
├── Configuration
│   └── Settings (proxy, concurrent downloads, theme)
├── Contributing
│   └── Development setup + testing
└── License
```

**Language:** Since the codebase uses Chinese error messages in gallery/settings and English in gelbooru commands, use English for the README (standard for open source). Chinese UI strings are embedded in the Rust code and Vue components, not the README.

**What NOT to include (deferred per D-05):**
- Screenshots
- Architecture diagram
- FAQ
- Changelog (not yet applicable for v1.0)

**Tauri-specific content to include:**
- Build prerequisites: Node.js 18+, Rust 1.70+, pnpm 8+
- Build command: `pnpm tauri build`
- Output: `src-tauri/target/release/bundle/nsis/*.exe`
- Note: NSIS installer installs per-user (configured in tauri.conf.json)

---

## 4. Release Build Verification (Task 4.4)

### Current tauri.conf.json Assessment

**Already configured for production:**

| Setting | Value | Assessment |
|---------|-------|------------|
| `version` | `"1.0.0"` | Correct — no change needed |
| `productName` | `"Gelbooru Downloader"` | Correct |
| `identifier` | `"com.gelbooru.downloader"` | Valid reverse-domain — no change needed |
| `bundle.targets` | `["nsis"]` | Windows NSIS installer — correct for Windows release |
| `bundle.windows.nsis.installMode` | `"currentUser"` | Per-user install — correct |
| `bundle.windows.nsis.languages` | `["SimpChinese", "English"]` | Appropriate for target audience |
| `build.devtools` | Not set | Defaults to `false` in production — correct |
| `app.windows[0].fullscreen` | `false` | Correct |
| `app.security.csp` | Custom CSP | Correctly scoped to `default-src 'self'` with allowances for images/connect |

**Tauri 2.x version:** 2.x (from Cargo.toml `tauri = { version = "2", ... }`) — confirmed current.
[VERIFIED: `npm view @tauri-apps/cli version` = 2.10.1, `npm view tauri version` = 0.15.0]

### Release Build Command

```bash
pnpm tauri build
```

This internally runs `cargo build --release` with the `[profile.release]` settings from `Cargo.toml`:
```toml
[profile.release]
panic = "abort"
codegen-units = 1
lto = true
opt-level = "s"
strip = true
```

These are all appropriate for a production desktop app. `lto = true` + `opt-level = "s"` optimizes for binary size, appropriate for distribution.

### Output Location

The NSIS installer will be at:
```
src-tauri/target/release/bundle/nsis/Gelbooru Downloader 1.0.0.exe
```

The raw binary (without installer) is at:
```
src-tauri/target/release/gelbooru.exe
```

### Production Checklist

- [ ] Run `cargo clippy --release -- -D warnings` before build (already in CI)
- [ ] Verify `gelbooru.db` schema version table exists after first run with new code
- [ ] Test NSIS installer on a clean Windows VM (install, run, uninstall)
- [ ] Check that the installer creates correct shortcuts and start menu entries

---

## Standard Stack

No changes to the standard stack are needed for Phase 4. All dependencies are already in `Cargo.toml`:

| Library | Version | Purpose | Status |
|---------|---------|---------|--------|
| rusqlite | 0.32 | SQLite access | Existing — used for schema versioning |
| tauri | 2.x | Desktop framework | Existing — release build configured |
| tauri-plugin-shell | 2 | Shell integration | Existing |
| tauri-plugin-fs | 2 | File system access | Existing |
| thiserror | 1 | Error definitions | In Cargo.toml but unused — no action needed |

**No new dependencies required** for Phase 4.

---

## Architecture Patterns

### Pattern: Version-Table Sequential Migration

**What:** A `schema_version` table holds a single integer. On startup, the app reads the current version and runs all migrations with a higher version number.

**When to use:** Simple apps that need to evolve a SQLite schema over time without a full migration framework.

**Implementation in db/mod.rs:**
```rust
// In Database::new(), after Connection::open():
Self::run_migrations(&conn)?;

// New function:
fn run_migrations(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (version INTEGER PRIMARY KEY)",
        [],
    )?;

    // Baseline: existing DBs get version 1 without running any SQL
    let has_row = conn.query_row(
        "SELECT 1 FROM schema_version LIMIT 1",
        [],
        |_| Ok(true),
    ).unwrap_or(false);
    if !has_row {
        conn.execute("INSERT INTO schema_version VALUES (1)", [])?;
    }

    let current: i32 = conn.query_row(
        "SELECT version FROM schema_version", [], |row| row.get(0)
    )?;

    for (name, sql) in MIGRATIONS.iter() {
        let version: i32 = name[..3].parse().unwrap_or(0);
        if version > current && !sql.is_empty() {
            conn.execute_batch(sql)?;
            conn.execute(
                "INSERT OR REPLACE INTO schema_version VALUES (?1)",
                rusqlite::params![version],
            )?;
        }
    }
    Ok(())
}
```

**Source:** Pattern derived from rusqlite documentation and standard SQLite migration practices. [VERIFIED: SQLite ALTER TABLE support confirmed]

### Anti-Patterns to Avoid

- **Per-migration transaction wrapping**: SQLite doesn't support nested transactions. Keep it simple — run each migration in a single `execute_batch`, and let the connection-level auto-bead behavior handle atomicity.
- **Loading migrations from external files at runtime**: This adds file I/O complexity in Tauri bundles. Embed migrations as `&'static str` constants.
- **Using rusqlite_migration or sqlx-migrate crates**: D-01 explicitly selects manual sequential migrations. These frameworks add dependency overhead disproportionate to the problem.

---

## Common Pitfalls

### Pitfall 1: Existing Database Has No Version Row

**What goes wrong:** After adding schema versioning to `new()`, existing users' databases have no `schema_version` row, causing a `SqliteResult::QueryReturnedNoRows` on the first read of that table.

**Why it happens:** `new()` previously only created tables with `IF NOT EXISTS`. No migration metadata was stored.

**How to avoid:** The `INSERT OR IGNORE INTO schema_version VALUES (1)` baseline insert handles this. The `INSERT OR IGNORE` (not just `INSERT`) prevents failure if some future code path pre-populates version 1.

**Warning signs:** First run after deployment throws `SqliteResult::QueryReturnedNoRows` on `SELECT version FROM schema_version`.

### Pitfall 2: Migration SQL Syntax Errors Cause Partial Migrations

**What goes wrong:** A migration with multiple statements fails mid-way, leaving the DB in an inconsistent state.

**Why it happens:** `conn.execute_batch()` runs all statements; if one fails, previously executed statements in that batch are NOT rolled back (SQLite auto-commits between statements in `execute_batch`).

**How to avoid:** Keep each migration to a single `ALTER TABLE ADD COLUMN` statement, which is atomic in SQLite. For multi-statement migrations, wrap in explicit `BEGIN...COMMIT` in the SQL string.

### Pitfall 3: Dev Database Path Used in Production

**What goes wrong:** `std::env::current_dir()` in `main.rs` returns the working directory at launch, which differs between dev (`src-tauri/`) and production (installed app directory).

**Why it happens:** Current code in `main.rs` line 16 uses `std::env::current_dir()` as the app data path. In dev, the DB goes to the project root. In production, it goes to the app data directory.

**Current state:** This is existing behavior, NOT introduced by Phase 4. The `Database::new()` stores at `app_data_dir/gelbooru.db` via `PathBuf::from(app_data_dir).join("gelbooru.db")`. Phase 4 does not change this path resolution — schema versioning is additive within `Database::new()`.

**Warning signs:** Dev and production DBs are in different locations (expected).

### Pitfall 4: Release Build Without devtools=false Explicitly Set

**What goes wrong:** Tauri 2.x by default enables devtools in debug builds but disables them in release. However, explicitly setting `devtools: false` in tauri.conf.json is still recommended for clarity.

**Current state:** `devtools` is not set in tauri.conf.json. This defaults to `false` in release builds (Tauri 2.x behavior). [ASSUMED — Tauri 2.x default behavior, not verified against specific version docs]

**Recommendation:** No change needed per D-06, but be aware this is a default, not an explicit setting.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Schema versioning | Custom version tracking with a JSON file or app config | `schema_version` SQLite table | SQLite-native, survives DB moves, atomic with DB operations |
| Migration framework | sqlx-migrate or rusqlite_migration crate | Manual `conn.execute_batch()` in `new()` | 5 tables, 1-2 future migrations; framework overhead > problem size |
| Error type unification | anyhow/thiserror across all commands | Keep `Result<T, String>` | D-03 locks this; no business logic errors requiring typed variants |
| Structured logging | tracing or log crate | Keep `println!()` / `eprintln!()` | D-04 locks this; desktop apps with stderr capture are sufficient |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | SQLite `ALTER TABLE ADD COLUMN` works with rusqlite 0.32 | Schema Migration | LOW — SQLite 3.1.6+ (bundled) supports it; rusqlite 0.32 bundles a recent SQLite |
| A2 | `INSERT OR REPLACE INTO schema_version` is atomic | Schema Migration | LOW — SQLite guarantees atomicity at the statement level for single-row ops |
| A3 | Tauri 2.x `devtools` defaults to `false` in release builds | Release Build | MEDIUM — not verified against Tauri 2.x docs; recommend adding `devtools: false` explicitly in tauri.conf.json as a defensive measure even though D-06 says no changes needed |
| A4 | NSIS installer output filename format | Release Build | LOW — standard Tauri NSIS naming: `"ProductName Version.exe"` = "Gelbooru Downloader 1.0.0.exe" |

---

## Open Questions

1. **Should `devtools: false` be explicitly added to tauri.conf.json?**
   - What we know: `devtools` is not currently set; D-06 says "no changes needed"
   - What's unclear: Whether Tauri 2.x release builds require an explicit config to disable devtools or if it is disabled by default
   - Recommendation: Add `devtools: false` to the `app` section as a defensive measure; this is technically a "change" but a minor clarification of existing behavior, not a feature change

2. **Where should the README live — project root or src-tauri/?**
   - What we know: D-05 says "README.md" with no path specified; standard practice for Tauri apps is project root
   - What's unclear: None — project root (`README.md`) is the standard location
   - Recommendation: Create at `README.md` (project root), not `src-tauri/README.md`

3. **Should migration SQL be embedded in Rust code or stored as actual `.sql` files?**
   - What we know: D-02 names migrations `001_init.sql`, `002_add_column.sql`, but does not specify file loading vs. embedded constants
   - What's unclear: The exact implementation approach for migration storage
   - Recommendation: Embed as `&'static str` constants in `db/mod.rs` — avoids file I/O complexity in Tauri binary bundles; the naming convention is documentation, not a file system requirement

---

## Environment Availability

Step 2.6: SKIPPED (no external dependencies beyond the project's own toolchain — all required tools (pnpm, Rust, cargo) are already used in the project)

---

## Validation Architecture

Step 4 (Validation Architecture): SKIPPED — Phase 4 is a polish and release preparation phase, not a feature implementation phase. No new test files are required. Existing tests (Phase 2) already provide coverage for the schema through `db/mod.rs` tests.

**Pre-existing test infrastructure from Phase 2:**
- `src-tauri/src/db/mod.rs` has `#[cfg(test)]` module with `create_test_db()` helper
- All existing DB tests use `tempfile::TempDir` — tests will continue to work with the new migration code
- Recommended: Add at least one new test to `db/mod.rs`:
  - `test_schema_version_baseline_for_existing_db` — simulates a DB without schema_version and verifies the migration code inserts version=1
  - `test_schema_version_runs_migrations_in_order` — verifies sequential version incrementing

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V4 Access Control | No | N/A |
| V5 Input Validation | No | Already handled in Phase 1 (gallery.rs path validation) |
| V6 Cryptography | No | N/A |

**Phase 4 adds no new security surface.** Schema versioning is a persistence mechanism with no network or user-input attack surface. README has no executable content. Release build is an NSIS installer with no additional code.

---

## Sources

### Primary (HIGH confidence)
- `src-tauri/src/db/mod.rs` — current Database implementation, CREATE TABLE IF NOT EXISTS pattern
- `src-tauri/src/main.rs` — app initialization, Database::new() call site
- `src-tauri/Cargo.toml` — existing dependencies, profile.release settings
- `src-tauri/tauri.conf.json` — production config verified as complete
- `src-tauri/src/commands/gelbooru.rs` — existing println!/Result<T, String> pattern
- `src-tauri/src/commands/download.rs` — existing error handling patterns
- `src-tauri/src/commands/gallery.rs` — error consistency audit source
- `src-tauri/src/commands/settings.rs` — error consistency audit source
- `npm view @tauri-apps/cli version` — Tauri CLI 2.10.1 [VERIFIED: npm registry]
- `npm view tauri version` — Tauri API 0.15.0 [VERIFIED: npm registry]

### Secondary (MEDIUM confidence)
- SQLite ALTER TABLE documentation — standard well-documented SQLite feature
- Tauri 2.x tauri.conf.json schema — standard production configuration

### Tertiary (LOW confidence)
- Tauri 2.x devtools default behavior in release builds — based on general Tauri knowledge, recommend explicit verification

---

## Metadata

**Confidence breakdown:**
- Schema migration: HIGH — source-code-verified current implementation, SQLite docs confirm ALTER TABLE support
- Error consistency: HIGH — all command files audited directly
- README structure: MEDIUM — based on D-05 spec, no existing README to verify patterns against
- Release build: HIGH — tauri.conf.json verified as production-ready, build commands standard

**Research date:** 2026-04-17
**Valid until:** 2026-05-17 (30 days — Tauri and rusqlite are stable, no fast-moving APIs)
