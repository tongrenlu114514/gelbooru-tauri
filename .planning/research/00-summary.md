# Research Summary

**Date:** 2026-04-14
**Type:** Brownfield Project Analysis

## Gelbooru API Analysis

### Authentication
- 使用浏览器 Cookie 认证
- Cookie 存储在 `cookie/com.gelbooru/cookies.json`
- 通过 reqwest CookieJar 导入

### API Pattern
- 非官方 API (无公开文档)
- 通过 HTML 抓取获取数据
- 使用 scraper 库解析 HTML
- 搜索: `https://gelbooru.com/index.php?page=post&s=list&tags=xxx`
- 分页: `&pid=xxx`

### Rate Limiting
- 当前无请求限流
- 可能需要添加延迟避免封禁

## Tauri 2.x Best Practices

### Security
- 使用 tauri-plugin-fs 进行文件操作
- 配置 CSP (Content Security Policy)
- 使用 asset protocol 安全地提供本地文件

### State Management
- Pinia stores for frontend state
- SQLite for persistent state
- Consider adding IPC for complex state sync

## Technical Debt Summary

### Must Fix (Before v2.0)
1. Settings persistence
2. Download task persistence
3. Hardcoded paths removal
4. Path sanitization

### Should Fix
1. Memory leak (imageCache)
2. Download retry mechanism
3. Test coverage

### Nice to Have
1. API rate limiting
2. Error handling improvements
3. Performance monitoring

## Dependencies Update Check

### Frontend (check for updates)
- vue: ^3.5.13
- naive-ui: ^2.41.0
- pinia: ^2.3.0
- vue-router: ^4.5.0
- @tauri-apps/api: ^2.2.0

### Backend (check for updates)
- tauri: 2.x
- rusqlite: 0.32
- reqwest: 0.12
- tokio: 1.x

## References

- Tauri 2.x Docs: https://v2.tauri.app/
- Vue 3 Docs: https://vuejs.org/
- Pinia Docs: https://pinia.vuejs.org/
