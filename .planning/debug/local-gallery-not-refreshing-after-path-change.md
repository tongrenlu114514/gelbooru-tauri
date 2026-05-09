---
status: resolved
trigger: "local-gallery-not-refreshing-after-path-change"
created: 2026-04-18T00:00:00.000Z
updated: 2026-04-18T00:00:00.000Z
---

## Current Focus
root_cause: FIXED. Three-part fix applied.
next_action: "Verify fix with manual test"

## Symptoms
expected: 设置新的下载路径后，本地图片库应立即刷新显示新路径下的图片
actual: 图片库没有自动刷新，仍然显示旧的路径内容
errors: 无报错信息
reproduction: 每次都可复现。步骤：打开设置 → 修改下载路径 → 点保存 → 回到本地库页面
started: 持续发生

## Eliminated

- **Gallery.vue reads from backend, not Pinia store** — `loadTree()` calls `invoke('get_directory_tree', {})` with no path, backend reads `download_path` from DB. This is correct. Bug is upstream.
- **Gallery has no watcher on settings store** — Secondary issue, addressed by adding watcher.
- **Debounce fires too late for navigation** — Root mechanism: debounce fires 500ms after last keystroke. But the "保存设置" button never triggered any backend save at all.

## Evidence
- timestamp: 2026-04-18
  checked: "Settings.vue saveSettings() function (lines 22-43)"
  found: "saveSettings() only calls localStorage.setItem() and invoke('set_proxy') — it NEVER calls invoke('save_settings') to persist download_path to the backend DB"
  implication: "Clicking '保存设置' does not save download_path to the DB. Backend's get_directory_tree reads stale value."
- timestamp: 2026-04-18
  checked: "settings.ts updateDownloadPath() and debounce mechanism"
  found: "updateDownloadPath() triggers saveSettingsDebounced() (500ms delay). User expects '保存设置' to save, but the button doesn't trigger any backend save."
  implication: "User types → debounce fires 500ms after last keystroke → navigates to Gallery → DB still has old path → Gallery loads stale tree."
- timestamp: 2026-04-18
  checked: "Gallery.vue loadTree()"
  found: "Calls invoke('get_directory_tree', {}) with no folder_path param. Backend reads from DB."
  implication: "Gallery is correct — it reads from DB. Bug is that DB is never updated on save."

## Resolution
root_cause: "Settings.vue's '保存设置' button calls a local saveSettings() that only writes to localStorage and proxy config, never persisting download_path to the backend. The debounced auto-save fires 500ms after last keystroke. When user clicks '保存设置' then immediately navigates to Gallery, the debounce may not have fired yet. Backend DB still has old path, Gallery loads stale tree. Additionally, Gallery had no watcher to react to downloadPath changes."
fix: "1. Added forceSave() to settings store — clears debounce timer and saves immediately. 2. Settings.vue save button now calls settingsStore.forceSave() to persist download_path to backend DB before navigating. 3. Added watch(() => settingsStore.downloadPath) in Gallery.vue to auto-refresh tree when path changes in store."
verification: "Manual test: open Settings → change download path → click 保存 → go to Gallery → verify new tree loads"
files_changed:
  - "src/stores/settings.ts — added forceSave() and exported it"
  - "src/views/Settings.vue — save button now calls settingsStore.forceSave()"
  - "src/views/Gallery.vue — added watch on settingsStore.downloadPath → calls refresh()"
