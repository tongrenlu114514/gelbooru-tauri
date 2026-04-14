# Phase 1: Foundation & Polish - Execution Plan

**Phase:** 01-foundation
**Created:** 2026-04-14
**Goal:** Fix critical issues and establish project foundation

## Tasks

### Task 1.1: 设置持久化到数据库
**Priority:** HIGH | **Files:** `settings.ts`, `db/mod.rs`

**Current State:**
- `settings.ts` 使用 `ref()` 存储在内存中
- 重启后所有设置重置为默认值

**Changes:**

1. **数据库层** (`src-tauri/src/db/mod.rs`):
   - 添加 `settings` 表:
     ```sql
     CREATE TABLE IF NOT EXISTS settings (
         key TEXT PRIMARY KEY,
         value TEXT NOT NULL
     );
     ```
   - 添加方法: `get_setting()`, `set_setting()`, `get_all_settings()`

2. **Rust Command** (`src-tauri/src/commands/settings.rs` - 新文件):
   - 创建 `get_settings` 命令: 返回所有设置
   - 创建 `save_settings` 命令: 保存设置
   - 创建 `load_settings` 命令: 从数据库加载设置

3. **前端 Store** (`src/stores/settings.ts`):
   - 添加 `loadSettings()` 方法，初始化时从后端加载
   - 添加 `saveSettings()` 方法，修改后自动保存
   - 使用 `debounce` 避免频繁保存

4. **移除硬编码默认值**:
   - 将 `downloadPath` 默认值从 `'D:/project/gelbooru/imgs/'` 改为从数据库读取
   - 如果数据库为空，使用系统默认路径 (通过 Tauri 获取)

**Success Criteria:**
- [ ] 设置修改后立即保存到数据库
- [ ] 重启后设置值正确恢复
- [ ] 无硬编码的默认值

---

### Task 1.2: 下载任务状态持久化
**Priority:** HIGH | **Files:** `download.rs`, `download.ts`, `db/mod.rs`

**Current State:**
- `downloads` 表已存在，但字段不完整
- 前端重启后任务列表为空
- `completed_at`, `error_message` 字段存在但未使用

**Changes:**

1. **数据库层** (`src-tauri/src/db/mod.rs`):
   - 更新 `downloads` 表结构（已完整）:
     ```sql
     CREATE TABLE IF NOT EXISTS downloads (
         id INTEGER PRIMARY KEY AUTOINCREMENT,
         post_id INTEGER NOT NULL,
         file_name TEXT NOT NULL,
         file_path TEXT NOT NULL,
         image_url TEXT NOT NULL,
         status TEXT NOT NULL DEFAULT 'pending',
         progress REAL NOT NULL DEFAULT 0,
         downloaded_size INTEGER NOT NULL DEFAULT 0,
         total_size INTEGER NOT NULL DEFAULT 0,
         created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
         completed_at TIMESTAMP,
         error_message TEXT
     );
     ```
   - 添加方法: `get_download_tasks()`, `update_download_progress()`, `get_pending_downloads()`

2. **Rust Command** (`src-tauri/src/commands/download.rs`):
   - 在 `add_download_task` 中正确持久化到数据库
   - 实现 `get_download_tasks` 命令，返回所有任务
   - 更新 `start_download`, `pause_download`, `cancel_download` 更新数据库状态
   - 实现 `resume_download` 命令

3. **前端 Store** (`src/stores/download.ts`):
   - 在 `init()` 或 `fetchTasks()` 中从后端加载所有任务
   - 确保重启后任务列表恢复

**Success Criteria:**
- [ ] 前端重启后任务列表恢复
- [ ] 下载进度实时保存
- [ ] 支持暂停/恢复下载

---

### Task 1.3: 路径清理和安全验证
**Priority:** HIGH | **Files:** `gallery.rs`

**Current State:**
- `gallery.rs` 中的命令直接使用传入的路径
- 无路径遍历防护
- `delete_image` 可以删除任意文件

**Changes:**

1. **创建路径工具模块** (`src-tauri/src/utils/path.rs` - 新文件):
   ```rust
   pub fn sanitize_path(path: &str, base_dir: &str) -> Result<PathBuf, PathError> {
       // 1. 规范化路径
       // 2. 解析相对路径
       // 3. 验证路径在 base_dir 内
       // 4. 返回规范化后的绝对路径
   }

   pub fn is_safe_path(path: &Path, allowed_base: &Path) -> bool {
       // 使用 canonicalize 解析真实路径
       // 检查是否在允许的目录内
   }
   ```

2. **更新 gallery.rs 命令**:
   - `delete_image`: 验证路径在下载目录内
   - `get_local_images`: 验证路径在允许范围内
   - `get_directory_tree`: 验证路径在允许范围内
   - `get_directory_images`: 验证路径在允许范围内
   - `get_local_image_base64`: 验证路径在允许范围内

3. **更新所有文件操作命令**:
   - 从数据库获取允许的基础路径
   - 使用路径验证工具

**Success Criteria:**
- [ ] 路径遍历攻击无效
- [ ] 所有文件操作在允许目录内
- [ ] 无路径注入漏洞

---

### Task 1.4: 移除硬编码路径
**Priority:** HIGH | **Files:** `gallery.rs`, `http.rs`, `settings.ts`

**Current State:**
- `gallery.rs:67,141`: `"D:/project/gelbooru/imgs/"`
- `settings.ts:7`: `'D:/project/gelbooru/imgs/'`
- `http.rs:15`: `"http://127.0.0.1:7897"` (代理)

**Changes:**

1. **gallery.rs**:
   - 移除第 67, 141 行的硬编码路径
   - 改为从设置中获取默认路径

2. **http.rs**:
   - 移除硬编码的代理地址
   - 从设置中读取代理配置

3. **settings.ts**:
   - 移除硬编码的下载路径
   - 使用 Tauri 获取系统默认下载目录

**Tauri 配置获取默认路径:**
```typescript
import { appDataDir } from '@tauri-apps/api/path'

// 在 loadSettings 中
const defaultPath = await appDataDir()
```

**Success Criteria:**
- [ ] `D:/project/gelbooru/imgs/` 不出现在代码中
- [ ] 应用在任意机器上运行
- [ ] 代理配置可动态设置

---

## Implementation Order

```
1. Task 1.1 (设置持久化) - 前置依赖
2. Task 1.4 (移除硬编码) - 依赖 1.1
3. Task 1.3 (路径安全)   - 依赖 1.4
4. Task 1.2 (下载持久化) - 独立，可并行
```

## Verification Checklist

- [ ] 设置修改后刷新页面值保留
- [ ] 重启应用后设置保留
- [ ] 下载任务重启后恢复
- [ ] 路径 `../../etc/passwd` 被拒绝
- [ ] 代码中无 `D:/project/gelbooru`
- [ ] `cargo clippy` 无警告
- [ ] `pnpm tsc` 无错误
