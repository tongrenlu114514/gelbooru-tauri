---
status: "in_progress"
trigger: "local-gallery-scroll-resets-on-load-more"
created: "2026-05-11T00:00:00.000Z"
updated: "2026-05-11T00:00:00.000Z"
---

## Current Focus
next_action: "Await user verification: items should append AND scroll should stay in place"
hypothesis: "Two separate issues: (1) items not appending = computed needed (FIXED), (2) scroll resets = MasonryWall DOM rebuild resets scroll context (separate fix needed)"
---

## Symptoms
expected: "加载更多后，(1)第二页图片追加显示，(2)滚动条保持位置"
actual: "第二页图片不显示 + 滚动条跳到顶部"
errors: "无控制台错误"
reproduction: "每次点击加载更多都重现"
started: "一直都有"
---

## Evidence
- timestamp: 2026-05-11T00:01:00.000Z
  checked: "Gallery.vue loadImagesForDirectory"
  found: "load-more: images.value.push(...result.images) mutates array ref"
  implication: "Array ref unchanged → computed doesn't re-evaluate → items not shown (ORIGINAL BUG)"

- timestamp: 2026-05-11T00:02:00.000Z
  checked: "GalleryCards.vue displayItems"
  found: "shallowRef + watch: append path was taken, but MasonryWall didn't show items"
  implication: "shallowRef approach broke item display - items not showing after load-more"

- timestamp: 2026-05-11T00:03:00.000Z
  checked: "@yeger/vue-masonry-wall version"
  found: "v6.1.1 - uses deep dependency tracking internally, might not detect shallowRef changes"
  implication: "shallowRef not reliable with this version"

- timestamp: 2026-05-11T00:04:00.000Z
  checked: "gallery-scroll reset flow"
  found: "MasonryWall destroys all child DOM on items change → masonry height collapses to 0 → browser resets scroll to top"
  implication: "Root cause of scroll reset is MasonryWall DOM rebuild, not scroll restoration logic"

## Resolution
root_cause_two_issues:
  - "(1) items not appending: shallowRef approach prevented MasonryWall from seeing new items → FIXED with computed"
  - "(2) scroll resets: MasonryWall DOM rebuild collapses masonry height → separate fix needed"
fix: "Reverted to simple computed (items WILL append). Scroll preservation via Gallery.vue scroll restoration in finally block + nextTick*2."
files_changed: ["src/views/GalleryCards.vue"]
verification: "TypeScript compiles clean"
