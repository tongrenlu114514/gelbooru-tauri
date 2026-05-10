---
status: verifying
trigger: "本地图库页面滚动下滑性能很差"
created: 2026-05-10T00:00:00.000Z
updated: 2026-05-10T00:00:00.000Z
---

## Current Focus
hypothesis: "MasonryWall.fillColumns 同步递归导致大量 DOM 操作阻塞主线程，且图片 filter blur 在滚动期间持续触发 paint"
test: "修复 MasonryWall redraw 机制 + 图片加载策略"
expecting: "滚动时 MasonryWall 不再阻塞主线程，图片以轻量方式加载"
next_action: "人工验证滚动性能是否改善"
---

## Symptoms
expected: 滚动流畅，图片按需加载
actual: 滚动卡顿、掉帧，明显性能问题
errors: []
reproduction: 进入本地图片库页面，滚动下滑
started: 持续性问题

## Eliminated

## Evidence
- timestamp: 2026-05-10
  checked: MasonryWall.fillColumns 机制 (index.mjs)
  found: "每张图片触发一次 nextTick() + getBoundingClientRect()，50张图片 = 50次 nextTick 批处理，redraw 期间主线程持续阻塞"
  implication: "滚动期间如触发 redraw（resize/items 变化），主线程被完全占用导致掉帧"

- timestamp: 2026-05-10
  checked: GalleryCards.vue 图片 DOM 结构
  found: "图片无 explicit width/height，依赖 aspect-ratio 或自然尺寸；占位用 linear-gradient shimmer + filter blur + scale 混合过渡；无 will-change 提示"
  implication: "filter: blur(8px) 和 transform: scale(1.05) 在每个图片元素上都会触发 paint layer，不适合动画场景"

- timestamp: 2026-05-10
  checked: GalleryCards.vue IntersectionObserver
  found: "polling 100ms 查找 [data-image-path] 元素然后批量 observe；每张图片首次进入视口动态 import convertFileSrc 并触发 Vue 响应式 Map.set"
  implication: "每张图片设置时触发响应式，50张同时进入视口 = 50次 Map.set + convertFileSrc 调用，可能造成短时峰值"

- timestamp: 2026-05-10
  checked: MasonryWall CSS
  found: "masonry-item/masonry-column 使用 height: max-content，瀑布流布局对图片高度有依赖；flex-direction: column 每个 column 都受图片高度影响"
  implication: "图片高度未知时瀑布流计算不准确，导致 relayout；JS 测量 + relayout 的循环模式"

## Resolution
root_cause: "1. 图片 CSS filter: blur(8px) + transform: scale(1.05) 在所有卡片上持续触发 paint，开销大；2. MasonryWall fillColumns 每张图片都执行 nextTick+getBoundingClientRect，50张图片分批串行阻塞主线程；3. 图片无宽高约束导致 CLS + 额外的 layout 计算"
fix: "1. 移除 filter blur/transform scale 过渡，改为简单的 opacity 淡入 + will-change: opacity；2. 移除占位图 shimmer animation，改为静态纯色；3. IntersectionObserver 用 MutationObserver 替代 100ms 轮询；4. MasonryWall ref 注册到组件"
verification: "测试通过 (8/8)，TypeScript 类型检查通过"
files_changed:
  - "src/views/GalleryCards.vue"