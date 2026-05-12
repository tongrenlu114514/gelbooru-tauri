# Phase 07: Image Viewer Enhancement - Validation

**Phase:** 07-image-viewer-enhancement
**Created:** 2026-05-13
**Type:** Validation Document

## Purpose

This document maps each requirement to its test specification and verification commands. It serves as the acceptance criteria for the phase gate.

## Test File References

Based on 07-RESEARCH.md Section "Validation Architecture" and "Wave 0 Gaps":

### Unit Test Files

| File | Tests | Command |
|------|-------|---------|
| `src/components/viewer/__tests__/zoom.test.ts` | Zoom behavior: wheel zoom, keyboard zoom, reset, clamping | `pnpm vitest run src/components/viewer/__tests__/zoom.test.ts` |
| `src/components/viewer/__tests__/pan.test.ts` | Pan/drag behavior: offset, cursor styles, drag activation | `pnpm vitest run src/components/viewer/__tests__/pan.test.ts` |
| `src/components/viewer/__tests__/keyboard.test.ts` | Keyboard shortcuts: arrows, escape, +/-, 0 key | `pnpm vitest run src/components/viewer/__tests__/keyboard.test.ts` |
| `src/components/viewer/__tests__/filmstrip.test.ts` | Filmstrip: visible range, active thumbnail, select emit | `pnpm vitest run src/components/viewer/__tests__/filmstrip.test.ts` |
| `src/components/viewer/__tests__/util.ts` | Test utilities: mock convertFileSrc, factory function | Shared by all viewer tests |

### E2E Test Files

| File | Tests | Command |
|------|-------|---------|
| `tests/viewer-fullscreen.spec.ts` | Fullscreen overlay, dark background on open (UI-01) | `pnpm exec playwright test tests/viewer-fullscreen.spec.ts` |
| `tests/viewer-navigation.spec.ts` | Navigate via buttons/keyboard (UI-02) | `pnpm exec playwright test tests/viewer-navigation.spec.ts` |
| `tests/viewer-keyboard.spec.ts` | Keyboard shortcuts: arrows, escape, +/-, 0 (UI-05) | `pnpm exec playwright test tests/viewer-keyboard.spec.ts` |

## Requirements -> Test Map

| Req ID | Description | Test File | Verification Command |
|--------|-------------|-----------|----------------------|
| UI-01 | Fullscreen modal overlay | `tests/viewer-fullscreen.spec.ts` | `playwright test tests/viewer-fullscreen.spec.ts` |
| UI-02 | Navigate via buttons/keyboard | `tests/viewer-navigation.spec.ts` | `playwright test tests/viewer-navigation.spec.ts` |
| UI-03 | Zoom via mouse wheel | `src/components/viewer/__tests__/zoom.test.ts` | `vitest run src/components/viewer/__tests__/zoom.test.ts` |
| UI-04 | Pan/drag zoomed image | `src/components/viewer/__tests__/pan.test.ts` | `vitest run src/components/viewer/__tests__/pan.test.ts` |
| UI-05 | Keyboard shortcuts | `tests/viewer-keyboard.spec.ts` | `playwright test tests/viewer-keyboard.spec.ts` |
| UI-06 | Filmstrip thumbnails | `src/components/viewer/__tests__/filmstrip.test.ts` | `vitest run src/components/viewer/__tests__/filmstrip.test.ts` |

## Verification Commands

### Per Task (during execution)

```bash
# After Task 1 (ImageViewer.vue)
pnpm vitest run src/components/viewer --passWithNoTests

# After Task 2 (Unit tests)
pnpm vitest run src/components/viewer/__tests__/zoom.test.ts
pnpm vitest run src/components/viewer/__tests__/pan.test.ts
pnpm vitest run src/components/viewer/__tests__/keyboard.test.ts

# After Task 3 (Checkpoint - build verification)
pnpm build

# Wave 2 - Filmstrip
pnpm vitest run src/components/viewer/__tests__/filmstrip.test.ts
```

### Per Wave (before merge)

```bash
# Wave 1 complete
pnpm vitest run src/components/viewer
pnpm build

# Wave 2 complete
pnpm vitest run src/components/viewer
pnpm test
```

### Phase Gate (before /gsd-verify-work)

```bash
# All tests must pass
pnpm vitest run src/components/viewer
pnpm exec playwright test
```

## Test Coverage Targets

| Component | Target Coverage |
|-----------|----------------|
| ImageViewer.vue | 80%+ line coverage |
| Filmstrip.vue | 80%+ line coverage |

## Success Criteria

- [ ] All unit tests pass: `pnpm vitest run src/components/viewer`
- [ ] All E2E tests pass: `pnpm exec playwright test`
- [ ] Build succeeds: `pnpm build`
- [ ] Type check passes: `pnpm tsc --noEmit`

## Wave 0 File Checklist

Based on 07-RESEARCH.md Wave 0 Gaps:

- [ ] `src/components/viewer/ImageViewer.vue` — main viewer component
- [ ] `src/components/viewer/Filmstrip.vue` — thumbnail strip
- [ ] `src/components/viewer/__tests__/zoom.test.ts` — zoom behavior
- [ ] `src/components/viewer/__tests__/pan.test.ts` — pan behavior
- [ ] `src/components/viewer/__tests__/filmstrip.test.ts` — filmstrip behavior
- [ ] `tests/viewer-fullscreen.spec.ts` — E2E fullscreen test
- [ ] `tests/viewer-navigation.spec.ts` — E2E navigation test
- [ ] `tests/viewer-keyboard.spec.ts` — E2E keyboard shortcuts