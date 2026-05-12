# Phase 7: Image Viewer Enhancement - Context

**Gathered:** 2026-05-13
**Status:** Ready for planning

<domain>
## Phase Boundary

Fullscreen image viewer with zoom/pan controls and keyboard navigation. Users can open images in fullscreen, zoom using mouse wheel or keyboard, pan around when zoomed, and navigate between images using buttons or keyboard arrows.

</domain>

<decisions>
## Implementation Decisions

### Zoom Controls (D-01)
- **Mouse wheel zoom**: Primary interaction for desktop users
- **Keyboard zoom**: +/- keys as secondary/accessibility option
- Both input methods work simultaneously

### Pan/Drag Behavior (D-02)
- **Auto pan when zoomed**: When image is zoomed in, cursor changes to grab state and dragging pans the image
- Pan mode activates automatically when zoom level > 100%
- No need to hold mouse button — click and drag works naturally

### Viewer Overlay (D-03)
- **Fullscreen overlay**: Covers entire screen for immersive viewing
- Similar to Lightroom's darkroom experience
- Background: dark/black overlay behind image

### Keyboard Navigation (D-04)
- **ArrowLeft/ArrowRight**: Navigate between images (already implemented in existing code)
- **Escape**: Close viewer
- **+/-**: Zoom in/out
- **0**: Reset zoom to fit

### Filmstrip (D-05)
- **Bottom filmstrip**: Shows neighboring images for quick navigation
- Thumbnail size: ~80px height, ~60px width (auto height)
- Display 7-9 thumbnails centered on current image
- Clicking thumbnail navigates to that image

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

No external specs — requirements fully captured in decisions above.

</canonical_refs>

<codebase_context>
## Existing Code Insights

### Reusable Assets
- `src/views/Gallery.vue`: Existing preview modal with NModal, keyboard navigation (ArrowLeft/Right/Escape), prev/next buttons, delete action
- `src/views/GalleryCards.vue`: Image display with card styling, hover states, convertFileSrc for asset URLs
- `convertFileSrc` from `@tauri-apps/api/core`: Used for rendering full-size images

### Established Patterns
- NModal for overlays (existing preview modal)
- Keyboard event handling with `window.addEventListener('keydown', handleKeydown)`
- Icon components from `@vicons/ionicons5`
- NButton with quaternary/circle variants for nav controls

### Integration Points
- `openPreview(index)` function in Gallery.vue opens modal
- `prevImage()`/`nextImage()` functions for navigation
- `handleKeydown()` function for keyboard handling
- `GalleryCards` component emits `open-preview` event

</codebase_context>

<specifics>
## Specific Ideas

- Fullscreen viewer like Lightroom darkroom mode
- Auto pan when zoomed — no need to hold mouse button
- Mouse wheel + keyboard (+/-) for zoom

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 07-image-viewer-enhancement*
*Context gathered: 2026-05-13*