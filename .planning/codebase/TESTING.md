# Testing Patterns

**Analysis Date:** 2026-04-13

## Current Testing Status

**No test framework is currently configured for this project.**

- No test files exist in the codebase
- No testing dependencies in `package.json`
- No testing dependencies in `Cargo.toml`

---

## Frontend Testing

### Recommended Framework

**Vitest** is the recommended testing framework for this Vue 3 + TypeScript project:

- Native ESM support
- Compatible with Vite
- Vue Test Utils integration
- TypeScript support out of the box

### Installation

```bash
pnpm add -D vitest @vue/test-utils happy-dom
```

### Configuration

Create `vite.config.ts`:

```typescript
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { fileURLToPath, URL } from 'node:url'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  },
  test: {
    globals: true,
    environment: 'happy-dom',
    include: ['src/**/*.{test,spec}.{js,ts}']
  }
})
```

Update `tsconfig.json`:

```json
{
  "compilerOptions": {
    "types": ["vitest/globals"]
  }
}
```

### Test File Organization

**Recommended structure:**

```
src/
├── components/
│   ├── AppSidebar.vue
│   └── AppSidebar.test.ts
├── views/
│   ├── Home.vue
│   └── Home.test.ts
├── stores/
│   ├── gallery.ts
│   └── gallery.test.ts
└── utils/
    ├── __tests__/
    │   └── format.test.ts
    └── index.ts
```

### Test Naming Conventions

```typescript
// src/stores/__tests__/gallery.test.ts
import { describe, it, expect, beforeEach } from 'vitest'

describe('useGalleryStore', () => {
  describe('setPosts', () => {
    it('should update posts state with new array', () => {
      // Test implementation
    })

    it('should replace existing posts', () => {
      // Test implementation
    })
  })

  describe('searchPosts', () => {
    it('should set loading to true during search', async () => {
      // Test implementation
    })
  })
})
```

### Example: Store Test

```typescript
// src/stores/__tests__/gallery.test.ts
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { createPinia, setActivePinia } from 'pinia'
import { describe, it, expect, beforeEach, vi } from 'vitest'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

describe('useGalleryStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('should initialize with empty posts', () => {
    const store = useGalleryStore()
    expect(store.posts).toHaveLength(0)
    expect(store.loading).toBe(false)
  })

  it('should set posts via setPosts action', () => {
    const store = useGalleryStore()
    const mockPosts = [{ id: 1, title: 'Test' }]

    store.setPosts(mockPosts)

    expect(store.posts).toEqual(mockPosts)
  })
})
```

### Example: Component Test

```typescript
// src/components/__tests__/AppSidebar.test.ts
import { mount } from '@vue/test-utils'
import { describe, it, expect } from 'vitest'
import AppSidebar from '../AppSidebar.vue'
import { createTestingPinia } from '@pinia/testing'

describe('AppSidebar', () => {
  it('should render navigation menu', () => {
    const wrapper = mount(AppSidebar, {
      global: {
        plugins: [createTestingPinia()]
      }
    })

    expect(wrapper.find('.n-menu').exists()).toBe(true)
  })

  it('should toggle sidebar on button click', async () => {
    const wrapper = mount(AppSidebar, {
      global: {
        plugins: [createTestingPinia({
          initialState: {
            settings: { sidebarCollapsed: false }
          }
        })]
      }
    })

    await wrapper.find('button').trigger('click')

    const settingsStore = useSettingsStore()
    expect(settingsStore.sidebarCollapsed).toBe(true)
  })
})
```

### Running Tests

```bash
# Run all tests
pnpm test

# Run tests in watch mode
pnpm test -- --watch

# Run with coverage
pnpm test -- --coverage

# Run specific file
pnpm test src/stores/gallery.test.ts
```

---

## Backend Testing (Rust)

### Test Framework

Rust uses built-in `#[test]` attribute with `#[cfg(test)]` modules.

### Test File Organization

```
src-tauri/src/
├── commands/
│   ├── mod.rs
│   ├── gelbooru.rs
│   └── tests/           # Integration tests
│       └── gelbooru_test.rs
├── models/
│   └── tests/
│       └── post_test.rs
└── services/
    └── tests/
        └── scraper_test.rs
```

### Unit Test Pattern

```rust
// src-tauri/src/models/post.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GelbooruPost {
    pub id: u32,
    pub url: String,
    pub title: String,
}

impl GelbooruPost {
    pub fn new(id: u32, url: String, title: String) -> Self {
        Self { id, url, title }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_post_with_correct_id() {
        let post = GelbooruPost::new(42, "http://example.com".into(), "Test".into());
        assert_eq!(post.id, 42);
    }

    #[test]
    fn creates_post_with_empty_tag_list() {
        let post = GelbooruPost::new(1, "url".into(), "title".into());
        assert!(post.tag_list.is_empty());
    }
}
```

### Integration Tests

Create `tests/` directory at project root:

```
src-tauri/tests/
├── commands_test.rs
└── scraper_test.rs
```

```rust
// src-tauri/tests/commands_test.rs

#[cfg(test)]
mod tests {
    #[test]
    fn test_search_posts_command() {
        // Integration test implementation
    }
}
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests in workspace root
cd src-tauri && cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_search_posts

# Run doc tests
cargo test --doc
```

---

## E2E Testing

### Recommended Framework

**Playwright** is recommended for this Tauri application:

```bash
pnpm add -D @playwright/test
npx playwright install
```

### E2E Test Structure

```
e2e/
├── example.spec.ts
└── fixtures/
    └── users.ts
```

### Example E2E Test

```typescript
// e2e/gallery.spec.ts
import { test, expect } from '@playwright/test'

test.describe('Gallery Page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    await page.click('text=搜索')
  })

  test('should display search results', async ({ page }) => {
    await page.fill('input[placeholder="输入标签搜索..."]', 'landscape')
    await page.click('button:has-text("搜索")')

    // Wait for results
    await expect(page.locator('.post-grid')).toBeVisible()
  })

  test('should open image preview', async ({ page }) => {
    await page.click('.post-card:first-child')

    await expect(page.locator('.preview-modal')).toBeVisible()
  })
})
```

### Running E2E Tests

```bash
# Open Playwright UI
pnpm playwright test --ui

# Run headless
pnpm playwright test

# Run specific test
pnpm playwright test e2e/gallery.spec.ts
```

---

## Test Coverage

### Frontend Coverage

```bash
# With vitest coverage
pnpm test -- --coverage

# Coverage thresholds (recommended minimum: 70%)
```

### Backend Coverage

```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Generate coverage report
cargo llvm-cov

# HTML report
cargo llvm-cov --html

# Fail if below threshold
cargo llvm-cov --fail-under-lines 70
```

---

## Mocking Patterns

### Tauri API Mocking

```typescript
// vitest.setup.ts
import { vi } from 'vitest'

export const mockInvoke = vi.fn()
export const mockListen = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
  convertFileSrc: vi.fn((path: string) => `asset://localhost/${path}`)
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: mockListen,
  UnlistenFn: vi.fn()
}))
```

### Store Testing with Pinia

```typescript
import { createPinia, setActivePinia } from 'pinia'
import { createTestingPinia } from '@pinia/testing'

// For unit tests
beforeEach(() => {
  setActivePinia(createPinia())
})

// For component tests
const wrapper = mount(Component, {
  global: {
    plugins: [createTestingPinia({
      initialState: {
        gallery: { posts: [], loading: false }
      }
    })]
  }
})
```

---

## Test Quality Checklist

Before marking code complete:
- [ ] Unit tests exist for new store actions
- [ ] Unit tests exist for utility functions
- [ ] Critical component rendering is tested
- [ ] Error paths are tested
- [ ] Edge cases are covered
- [ ] No hardcoded test data in production code

---

## CI Integration

### GitHub Actions Example

```yaml
# .github/workflows/test.yml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup pnpm
        uses: pnpm/action-setup@v2

      - name: Install dependencies
        run: pnpm install

      - name: Type check
        run: pnpm vue-tsc --noEmit

      - name: Run tests
        run: pnpm test --coverage

      - name: Rust tests
        run: cd src-tauri && cargo test
```

---

*Testing analysis: 2026-04-13*
