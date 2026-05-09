---
status: resolved
trigger: "download-record-reappears-after-restart"
created: 2026-04-18T00:00:00Z
updated: 2026-04-18T00:00:00Z
---

## Current Focus
root_cause_confirmed: "remove_download_task only removes from in-memory DOWNLOAD_MANAGER HashMap, not from SQLite. On restart, restore_download_tasks reloads all DB records including the 'deleted' ones."
next_action: "Fix remove_download_task to call database.delete_download_task(id) and fix clear_completed_tasks similarly"

## Symptoms
expected: "删除的下载记录应永久删除，重启后不再出现"
actual: "删除的下载记录在重启应用后重新出现"
errors: "无报错信息"
reproduction: "删除下载记录 → 重启应用 → 记录重新出现"
started: "持续发生"

## Eliminated
<!-- APPEND ONLY -->

## Evidence
- timestamp: 2026-04-18
  checked: "src-tauri/src/commands/download.rs remove_download_task (line 564)"
  found: "Only calls DOWNLOAD_MANAGER.remove_task(id) — removes from in-memory HashMap only. Does NOT delete from SQLite."
  implication: "Record persists in DB, reappears after restart."
- timestamp: 2026-04-18
  checked: "src-tauri/src/db/mod.rs delete_download_task (line 450)"
  found: "Database method exists: DELETE FROM downloads WHERE id = ?1 — properly removes from SQLite."
  implication: "The fix is to call this method from remove_download_task."
- timestamp: 2026-04-18
  checked: "src-tauri/src/commands/download.rs restore_download_tasks (line 578)"
  found: "On startup, loads ALL records from database via get_all_download_tasks() — including records that were 'deleted' from memory."
  implication: "Without DB-level deletion, deleted tasks reappear on restart."
- timestamp: 2026-04-18
  checked: "src-tauri/src/commands/download.rs clear_completed_tasks (line 605)"
  found: "Same bug: only removes from in-memory manager, not from SQLite."
  implication: "Cleared completed tasks also reappear after restart."

## Resolution
root_cause: "remove_download_task and clear_completed_tasks only remove records from the in-memory DOWNLOAD_MANAGER HashMap. Neither command calls database.delete_download_task(), so records remain in SQLite. On restart, restore_download_tasks loads all DB records back into memory."
fix: "Added database.delete_download_task(id) call to remove_download_task (line 567) and clear_completed_tasks (line 612), mirroring the existing pattern used by add_download_task for persistence."
verification: "cargo check passes. Manual test: delete a download record → restart app → record should not reappear."
files_changed:
  - "src-tauri/src/commands/download.rs — added database.delete_download_task() calls to remove_download_task and clear_completed_tasks"
