# Gelbooru Downloader

## 项目概述

一个基于 **Tauri 2.0 + Vue 3** 开发的 Gelbooru 图片下载器桌面应用，提供图片搜索、浏览、下载和管理功能。

### 主要技术栈

**前端**
- **框架**: Vue 3.5 + TypeScript 5.7
- **构建工具**: Vite 6.0
- **UI 组件库**: Naive UI 2.41
- **状态管理**: Pinia 2.3
- **路由**: Vue Router 4.5
- **图标**: @vicons/ionicons5

**后端 (Rust)**
- **框架**: Tauri 2.0
- **HTTP 客户端**: reqwest 0.12 (支持 rustls-tls)
- **HTML 解析**: scraper 0.22
- **数据库**: rusqlite 0.32 (SQLite)
- **异步运行时**: tokio 1.x (full features)
- **序列化**: serde + serde_json
- **其他**: chrono, futures, regex, thiserror, lazy_static, base64

### 项目架构

```
gelbooru/
├── src/                          # Vue 前端源码
│   ├── App.vue                   # 根组件 (布局 + 主题)
│   ├── main.ts                   # 应用入口
│   ├── components/
│   │   ├── AppSidebar.vue        # 侧边栏导航
│   │   └── DownloadNotifier.vue  # 下载通知组件
│   ├── router/
│   │   └── index.ts              # 路由配置
│   ├── stores/                   # Pinia 状态管理
│   │   ├── download.ts           # 下载任务状态
│   │   ├── favoriteTags.ts       # 收藏标签状态
│   │   ├── gallery.ts            # 图库状态
│   │   └── settings.ts           # 应用设置
│   ├── types/
│   │   └── index.ts              # TypeScript 类型定义
│   └── views/                    # 页面组件
│       ├── Home.vue              # 首页 (搜索)
│       ├── Downloads.vue         # 下载管理
│       ├── Gallery.vue           # 本地图库
│       ├── FavoriteTags.vue      # 收藏标签
│       └── Settings.vue          # 设置页面
│
├── src-tauri/                    # Rust 后端源码
│   ├── main.rs                   # Tauri 入口 + 命令注册
│   ├── lib.rs                    # 模块声明 (空)
│   ├── commands/                 # Tauri 命令
│   │   ├── mod.rs                # 模块导出
│   │   ├── gelbooru.rs           # Gelbooru API 命令
│   │   ├── download.rs           # 下载管理命令
│   │   ├── gallery.rs            # 图库管理命令
│   │   └── favorite_tags.rs      # 收藏标签命令
│   ├── models/                   # 数据模型
│   │   ├── mod.rs
│   │   ├── post.rs               # 帖子模型
│   │   ├── tag.rs                # 标签模型
│   │   └── page.rs               # 分页模型
│   ├── services/                 # 服务层
│   │   ├── mod.rs
│   │   ├── http.rs               # HTTP 客户端
│   │   └── scraper.rs            # HTML 解析器
│   ├── db/
│   │   └── mod.rs                # SQLite 数据库操作
│   └── gelbooru.db               # SQLite 数据库文件
│
├── cookie/                       # Cookie 存储 (遗留)
├── dist/                         # 前端构建输出
└── target/                       # Rust 构建输出
```

## 核心功能

### 1. 图片搜索
- 按标签搜索 Gelbooru 图片
- 自动过滤视频和动画 (`-video`, `-animated` 标签排除)
- 优先抓取高分辨率图片 (`highres` 标签)
- 每页 42 张图片，支持分页遍历

### 2. 下载管理
- 多任务并发下载 (默认 3 个并发)
- 支持暂停、恢复、取消操作
- 实时进度显示
- 智能文件命名和分类存储

### 3. 智能分类存储
下载的图片按以下结构自动分类存储：
```
{downloadPath}/
└── {postedDate}/                    # 发布日期
    └── {rating}/                     # 内容分级 (safe/questionable/explicit)
        └── {copyright}/               # 版权/作品名
            └── [{character}]{id}({artist}).{ext}  # 文件名
```

### 4. 本地图库
- 浏览已下载的图片
- 按目录树导航
- 支持删除图片

### 5. 收藏标签
- 收藏常用标签
- 支持父子层级分组
- 快速搜索收藏的标签

### 6. 应用设置
- 主题切换 (亮色/暗色)
- 下载路径配置
- 并发下载数设置
- 代理配置

## 构建与运行

### 环境要求
- Node.js 18+
- pnpm 8+
- Rust 1.70+
- Tauri CLI 2.0

### 开发模式

```bash
# 安装依赖
pnpm install

# 启动开发服务器
pnpm tauri dev
```

### 构建发布

```bash
# 构建生产版本
pnpm tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`。

### 仅前端开发

```bash
# 启动 Vite 开发服务器
pnpm dev

# 构建前端
pnpm build

# 预览构建结果
pnpm preview
```

## Tauri 命令 API

### Gelbooru 命令

| 命令 | 参数 | 说明 |
|------|------|------|
| `search_posts` | `tags: string[]`, `page: number` | 搜索帖子 |
| `get_post_detail` | `id: number` | 获取帖子详情 |
| `get_image_base64` | `url: string` | 获取图片 Base64 |
| `set_proxy` | `proxy_url: string \| null` | 设置代理 |

### 下载命令

| 命令 | 参数 | 说明 |
|------|------|------|
| `add_download_task` | `postId`, `imageUrl`, `fileName`, `savePath` | 添加下载任务 |
| `start_download` | `id: number` | 开始下载 |
| `pause_download` | `id: number` | 暂停下载 |
| `resume_download` | `id: number` | 恢复下载 |
| `cancel_download` | `id: number` | 取消下载 |
| `remove_download_task` | `id: number` | 移除任务 |
| `get_download_tasks` | - | 获取所有任务 |
| `clear_completed_tasks` | - | 清除已完成任务 |
| `open_file` | `path: string` | 打开文件 |

### 图库命令

| 命令 | 参数 | 说明 |
|------|------|------|
| `get_local_images` | `dir: string` | 获取本地图片 |
| `delete_image` | `path: string` | 删除图片 |
| `get_directory_tree` | `basePath: string` | 获取目录树 |
| `get_directory_images` | `dir: string` | 获取目录图片 |

### 收藏标签命令

| 命令 | 参数 | 说明 |
|------|------|------|
| `get_favorite_tags` | - | 获取收藏标签 |
| `add_parent_tag` | `tag`, `tagType` | 添加父标签 |
| `add_child_tag` | `tag`, `tagType`, `parentId` | 添加子标签 |
| `remove_favorite_tag` | `id: number` | 移除收藏标签 |
| `is_tag_favorited` | `tag: string` | 检查是否已收藏 |
| `get_child_tags` | `parentId: number` | 获取子标签 |

## 数据模型

### GelbooruPost
```typescript
interface GelbooruPost {
  id: number
  url: string
  title: string
  tagList: GelbooruTag[]
  statistics: GelbooruPostStatistics
  thumbnail?: string
}
```

### GelbooruTag
```typescript
interface GelbooruTag {
  text: string
  tagType: string  // artist, character, copyright, general
  count: number
}
```

### DownloadTask
```typescript
interface DownloadTask {
  id: number
  postId: number
  imageUrl: string
  fileName: string
  savePath: string
  status: 'pending' | 'downloading' | 'completed' | 'failed' | 'paused' | 'cancelled'
  progress: number
  totalSize: number
  downloadedSize: number
  error?: string
}
```

## 数据库结构

数据库文件: `src-tauri/gelbooru.db`

### 表结构

**downloads** - 下载记录
```sql
id, post_id, file_name, file_path, image_url, status, 
progress, downloaded_size, total_size, created_at, completed_at, error_message
```

**favorites** - 收藏帖子
```sql
id, post_id, created_at
```

**favorite_tags** - 收藏标签
```sql
id, tag, tag_type, parent_id, created_at
```

**blacklisted_tags** - 黑名单标签
```sql
id, tag, created_at
```

## 开发规范

### 前端代码风格
- 使用 Vue 3 Composition API (`<script setup>`)
- 使用 Pinia 进行状态管理
- 使用 TypeScript 类型检查
- 组件命名: PascalCase
- 文件命名: PascalCase.vue

### 后端代码风格
- 使用 `#[tauri::command]` 宏暴露命令
- 异步操作使用 `async/await`
- 错误处理返回 `Result<T, String>`
- 使用 `lazy_static!` 管理全局状态

### 事件通信
前端监听后端事件:
```typescript
import { listen } from '@tauri-apps/api/event'

listen<DownloadProgressEvent>('download-progress', (event) => {
  // 处理下载进度
})
```

## 注意事项

1. **代理设置**: 默认代理 `127.0.0.1:7897`，可在设置页面配置
2. **CSP 配置**: 已配置允许加载 HTTPS 图片资源
3. **并发控制**: 下载任务通过 Semaphore 控制并发数
4. **断点续传**: 暂停时保留临时文件，恢复时重新下载

## 配置文件

### tauri.conf.json
- 窗口尺寸: 1200x800 (最小 800x600)
- CSP 安全策略已配置
- 支持 Windows/macOS/Linux 三平台

### tsconfig.json
- 目标: ESNext
- 模块: ESNext
- 严格模式启用

---
*最后更新: 2026-03-26*