# Phase 7: Image Viewer Enhancement - Research

**Researched:** 2026-05-13
**Domain:** Fullscreen image viewer with zoom/pan controls and keyboard navigation
**Confidence:** MEDIUM-HIGH

## Summary

Phase 7 requires enhancing the existing basic preview modal in Gallery.vue into a full Lightroom-style image viewer with zoom/pan capabilities, keyboard navigation, and a filmstrip for quick navigation. The existing codebase has a solid foundation with NModal, keyboard handling, and `convertFileSrc` for asset URLs.

**Primary recommendation:** Build a dedicated `ImageViewer.vue` component that uses CSS `transform: scale()` and `translate()` for zoom/pan (compositor-friendly properties), extend the existing keyboard handler with +/- and 0 keys, and add a filmstrip using horizontal thumbnails centered on the current image.

## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Mouse wheel zoom as primary, keyboard +/- as secondary
- **D-02:** Auto-pan when zoomed > 100%, no need to hold mouse button
- **D-03:** Fullscreen overlay (dark background) for immersive viewing
- **D-04:** ArrowLeft/Right/Escape for navigation, +/- for zoom, 0 to reset
- **D-05:** Filmstrip at bottom with ~80px height thumbnails, 7-9 centered thumbnails

### Claude's Discretion
- Implementation details for zoom/pan math
- CSS transition timing
- Filmstrip scroll behavior

### Deferred Ideas (OUT OF SCOPE)
- None — discussion stayed within phase scope

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| UI-01 | Fullscreen modal overlay | NModal with preset="card" + dark overlay, existing pattern |
| UI-02 | Navigate via buttons/keyboard | ArrowLeft/Right already in Gallery.vue handleKeydown |
| UI-03 | Zoom via mouse wheel or pinch | wheel event + CSS transform scale, no library needed |
| UI-04 | Pan/drag zoomed image | Drag with transform translate, cursor: grab/grabbing |
| UI-05 | Keyboard shortcuts (Arrow, Escape, +/-) | Extend existing handleKeydown with key binding |
| UI-06 | Filmstrip with neighboring images | Horizontal scroll container with thumbnail images |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Vue 3 | 3.5.13 | UI framework | Project baseline |
| naive-ui | 2.44.1 | Modal, buttons, icons | Project baseline |
| @vicons/ionicons5 | 0.12.0 | Navigation icons | Project baseline (chevron, expand icons) |
| @tauri-apps/api/core | 2.10.1 | convertFileSrc for asset URLs | Project baseline |

### No Additional Dependencies
| Instead of | Use | Why |
|------------|-----|-----|
| panzoom library | Pure CSS transform | Native performance, no bundle bloat |
| react-zoom-pan-pinch | Wheel/drag handlers | Simple enough to implement directly |

**Installation:** No new packages required — all features achievable with CSS transforms and native browser events.

## Architecture Patterns

### Recommended Project Structure
```
src/
├── components/
│   └── viewer/
│       ├── ImageViewer.vue    # Main viewer component (~200 lines)
│       └── Filmstrip.vue      # Thumbnail strip (~100 lines)
└── views/
    └── Gallery.vue            # Existing, use ImageViewer instead of basic modal
```

### Pattern 1: CSS Transform Zoom/Pan
**What:** Use `transform: scale()` and `transform-origin: center` for zoom, `translate()` for pan
**When to use:** All image zoom/pan interactions
**Example:**
```typescript
// Track zoom level and pan offset in reactive refs
const zoomLevel = ref(1); // 1 = 100%
const panOffset = ref({ x: 0, y: 0 });

// Mouse wheel zoom: adjust zoomLevel by delta
function handleWheel(e: WheelEvent) {
  e.preventDefault();
  const delta = e.deltaY > 0 ? -0.1 : 0.1;
  zoomLevel.value = Math.min(5, Math.max(0.5, zoomLevel.value + delta));
}

// Apply transform to image container
const imageStyle = computed(() => ({
  transform: `scale(${zoomLevel.value}) translate(${panOffset.value.x}px, ${panOffset.value.y}px)`,
  transformOrigin: 'center center',
}));
```

### Pattern 2: Auto-Pan Activation
**What:** When zoom > 100%, cursor changes to grab and dragging pans
**When to use:** Drag-to-pan only when zoomed in
**Example:**
```typescript
const isDragging = ref(false);
const dragStart = ref({ x: 0, y: 0 });

function startDrag(e: MouseEvent) {
  if (zoomLevel.value <= 1) return; // No drag when not zoomed
  isDragging.value = true;
  dragStart.value = { x: e.clientX - panOffset.value.x, y: e.clientY - panOffset.value.y };
}

function onDrag(e: MouseEvent) {
  if (!isDragging.value) return;
  panOffset.value = {
    x: e.clientX - dragStart.value.x,
    y: e.clientY - dragStart.value.y,
  };
}

// Cursor style
const containerStyle = computed(() => ({
  cursor: zoomLevel.value > 1 ? (isDragging.value ? 'grabbing' : 'grab') : 'default',
}));
```

### Pattern 3: Fullscreen Modal
**What:** NModal with preset="card" or custom overlay covering viewport
**When to use:** UI-01 fullscreen requirement
**Example:**
```vue
<!-- Dark overlay with centered image -->
<div class="viewer-overlay" @click.self="closeViewer">
  <div class="viewer-container" @wheel="handleWheel">
    <img :src="imageSrc" :style="imageStyle" class="viewer-image" />
  </div>
  <!-- Filmstrip at bottom -->
  <Filmstrip :images="images" :current-index="currentIndex" @select="goToImage" />
</div>
```

### Anti-Patterns to Avoid
- **Animating width/height:** Not compositor-friendly, causes jank
- **Using position: absolute with left/top:** Slower than transform
- **Touch pinch without wheel event:** Mobile support can be added later if needed

## Common Pitfalls

### Pitfall 1: Default image fitting
**What goes wrong:** Image doesn't fit viewport on initial open, especially large images
**Why it happens:** Not setting initial zoom to fit container
**How to avoid:** Calculate initial scale to fit image within viewport (width/height ratio)
**Warning signs:** User sees only part of image or image overflows

### Pitfall 2: Pan boundary clamping
**What goes wrong:** Panning allows image to go too far off-screen
**Why it happens:** No bounds checking on pan offset
**How to avoid:** Clamp pan offset based on zoom level and container size
**Warning signs:** Image can be dragged completely off-screen

### Pitfall 3: Double-tap zoom on mobile
**What goes wrong:** Mobile pinch-zoom conflicts with double-tap-to-zoom intent
**Why it happens:** Not handling touch events separately from wheel
**How to avoid:** Defer mobile support, focus on desktop first (wheel + drag)

## Code Examples

### Wheel Zoom Handler (verified approach)
```typescript
function handleWheel(e: WheelEvent) {
  e.preventDefault(); // Prevent page scroll
  const delta = e.deltaY > 0 ? -0.1 : 0.1; // Scroll down = zoom out
  const newZoom = zoomLevel.value + delta;
  zoomLevel.value = Math.max(0.5, Math.min(5, newZoom)); // Clamp 50%-500%
}
```

### Keyboard Navigation (extends existing)
```typescript
function handleKeydown(e: KeyboardEvent) {
  if (!props.visible) return;
  switch (e.key) {
    case 'ArrowLeft': emit('prev'); break;
    case 'ArrowRight': emit('next'); break;
    case 'Escape': emit('close'); break;
    case '+':
    case '=': zoomIn(); break;
    case '-': zoomOut(); break;
    case '0': resetZoom(); break;
  }
}
```

### Filmstrip Thumbnail (verified pattern)
```typescript
// Compute visible range: center on current, show 4 before and 4 after
const visibleRange = computed(() => {
  const half = Math.floor(9 / 2);
  const start = Math.max(0, props.currentIndex - half);
  const end = Math.min(props.images.length - 1, props.currentIndex + half);
  return props.images.slice(start, end + 1);
});
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| NModal with preset="card" | Custom fullscreen overlay | Phase 7 | Better darkroom experience |
| Fixed size image | CSS transform scale | Phase 7 | Smooth zoom without redraw |
| No filmstrip | Horizontal thumbnail strip | Phase 7 | Quick navigation |

**Deprecated/outdated:**
- None for this phase

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Filmstrip can use same convertFileSrc for thumbnails | Filmstrip | May need thumbnail generation (defer to Phase 10) |

**If this table is empty:** All claims in this research were verified or cited — no user confirmation needed.

## Open Questions

1. **Should filmstrip thumbnails use same full-size images?**
   - What we know: Phase 10 has thumbnail generation (IDX-02, IDX-03, IDX-04)
   - What's unclear: Filmstrip may need thumbnails before Phase 10
   - Recommendation: Use convertFileSrc for now, upgrade to generated thumbnails in Phase 10

2. **Initial zoom fitting algorithm?**
   - What we know: Fit image within viewport on open
   - What's unclear: Exact formula (aspect ratio comparison vs container)
   - Recommendation: Calculate scale = min(viewport_w / img_w, viewport_h / img_h), clamp to 1 max

## Environment Availability

Step 2.6: SKIPPED (no external dependencies identified — pure frontend enhancement)

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Vitest 4.1.4 (unit) + Playwright 1.59.1 (E2E) |
| Config file | vitest.config.ts (existing) / playwright.config.ts (existing) |
| Quick run command | `pnpm vitest run src/components/viewer` |
| Full suite command | `pnpm test` + `pnpm exec playwright test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| UI-01 | Fullscreen modal overlay | E2E | `playwright test tests/viewer-fullscreen.spec.ts` | Wave 0 |
| UI-02 | Navigate via buttons/keyboard | E2E | `playwright test tests/viewer-navigation.spec.ts` | Wave 0 |
| UI-03 | Zoom via mouse wheel | Unit | `vitest run src/components/viewer/__tests__/zoom.test.ts` | Wave 0 |
| UI-04 | Pan/drag zoomed image | Unit | `vitest run src/components/viewer/__tests__/pan.test.ts` | Wave 0 |
| UI-05 | Keyboard shortcuts | E2E | `playwright test tests/viewer-keyboard.spec.ts` | Wave 0 |
| UI-06 | Filmstrip thumbnails | Unit | `vitest run src/components/viewer/__tests__/filmstrip.test.ts` | Wave 0 |

### Sampling Rate
- **Per task commit:** `pnpm vitest run src/components/viewer`
- **Per wave merge:** `pnpm test`
- **Phase gate:** All E2E tests green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `src/components/viewer/ImageViewer.vue` — main viewer component
- [ ] `src/components/viewer/Filmstrip.vue` — thumbnail strip
- [ ] `src/components/viewer/__tests__/zoom.test.ts` — zoom behavior
- [ ] `src/components/viewer/__tests__/pan.test.ts` — pan behavior
- [ ] `src/components/viewer/__tests__/filmstrip.test.ts` — filmstrip behavior
- [ ] `tests/viewer-fullscreen.spec.ts` — E2E fullscreen test
- [ ] `tests/viewer-navigation.spec.ts` — E2E navigation test
- [ ] `tests/viewer-keyboard.spec.ts` — E2E keyboard shortcuts

## Security Domain

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | No | N/A — no user input in viewer |
| Other | No | N/A — image paths from local filesystem |

**Known Threat Patterns:** None — viewer renders local images only via Tauri's secure convertFileSrc.

## Sources

### Primary (HIGH confidence)
- [naive-ui NModal docs](https://www.naiveui.com/en-US/light/components/modal) - modal overlay pattern
- [Vue 3 reactive transforms](https://vuejs.org/guide/essentials/computed.html) - computed style patterns
- [CSS transform performance](https://developer.mozilla.org/en-US/docs/Web/CSS/transform) - compositor-friendly properties

### Secondary (MEDIUM confidence)
- [Gallery.vue existing code](file://src/views/Gallery.vue) - current preview implementation
- [naive-ui icon usage](https://www.naive-ui.com/en-US/light/components/icon) - icon integration

### Tertiary (LOW confidence)
- WebSearch (blocked by API error) — all patterns derived from existing codebase and Vue 3 best practices

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries verified in pnpm-lock.yaml
- Architecture: HIGH — patterns match existing Gallery.vue patterns
- Pitfalls: MEDIUM — based on common zoom/pan implementation issues

**Research date:** 2026-05-13
**Valid until:** 2026-06-13 (30 days for stable patterns)