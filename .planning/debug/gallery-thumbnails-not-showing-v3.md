---
status: investigating
trigger: "缩略图不会显示（白色占位图）"
created: 2026-05-10T00:00:00.000Z
updated: 2026-05-10T00:00:00.000Z
---

## Current Focus
hypothesis: "startSelfObserver() returns null and doesn't set selfObserver when MasonryWall not yet rendered, then watches[props.images] doesn't call startSelfObserver again with correct timing (Vue flush order)"
test: "Run Tauri app in dev mode with Playwright to observe actual DOM and network behavior"
expecting: "Either observer fires (which means logic is correct, problem is URL/network) OR observer never fires (logic bug)"
next_action: "Build Tauri app and use Playwright to open the built .exe, inspect DOM img elements and network requests"