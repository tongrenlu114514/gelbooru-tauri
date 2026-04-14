# Gelbooru Downloader - Requirements

**Status:** Based on existing codebase v1.0.0

## Existing Features

### Core Features
- [x] 图片搜索 (标签搜索 Gelbooru)
- [x] 图片下载 (并发下载控制)
- [x] 本地图库 (浏览本地图片)
- [x] 收藏标签 (管理常用标签)
- [x] 设置管理 (下载路径、代理配置)

### Technical Features
- [x] Tauri 2.x 桌面应用
- [x] SQLite 本地数据库
- [x] Cookie 认证
- [x] 响应式 UI (naive-ui)

## Identified Issues (from codebase analysis)

### Security
- [ ] 硬编码路径需移除
- [ ] 添加路径清理防止路径遍历
- [ ] 添加 API 请求限流

### Persistence
- [ ] 设置持久化 (当前重启后丢失)
- [ ] 下载任务持久化 (当前重启后丢失)
- [ ] 数据库 schema 版本管理

### Performance
- [ ] 解决 imageCache 内存泄漏
- [ ] 大目录扫描优化
- [ ] 添加下载重试机制

### Testing
- [ ] 添加单元测试
- [ ] 添加集成测试
- [ ] 配置 CI/CD

## Potential Enhancements

### High Priority
1. 设置持久化到数据库
2. 下载任务状态持久化
3. 路径清理和安全验证
4. 添加基础测试框架

### Medium Priority
1. 下载重试机制
2. 大目录扫描优化
3. 内存缓存清理机制
4. ESLint/Prettier 配置

### Low Priority
1. API 限流
2. 更多错误处理
3. 性能监控
4. 用户操作日志
