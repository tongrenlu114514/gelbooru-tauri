import { vi } from 'vitest';

// ---------------------------------------------------------------------------
// Module-level spies — must be declared BEFORE vi.mock calls (hoisting order)
// ---------------------------------------------------------------------------
const invokeMock = vi.fn();
const convertFileSrcMock = vi.fn((path: string) => `mock://asset/${path}`);
const listenMock = vi.fn().mockResolvedValue(vi.fn());

const matchMediaMock = vi.fn().mockImplementation((query) => ({
  matches: false,
  media: query,
  onchange: null,
  addListener: vi.fn(),
  removeListener: vi.fn(),
  addEventListener: vi.fn(),
  removeEventListener: vi.fn(),
  dispatchEvent: vi.fn(),
}));

// ---------------------------------------------------------------------------
// ResizeObserver mock using class syntax (required for vueuc compatibility)
// ---------------------------------------------------------------------------
class FakeResizeObserver {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

// ---------------------------------------------------------------------------
// Tauri API mocks — vi.mock is hoisted; all referenced variables must exist
// ---------------------------------------------------------------------------
vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
  convertFileSrc: convertFileSrcMock,
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: listenMock,
  type: { UnlistenFn: 'UnlistenFn' },
}));

// ---------------------------------------------------------------------------
// Browser API mocks — set directly on global/window (no hoisting needed)
// ---------------------------------------------------------------------------
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: matchMediaMock,
});

global.ResizeObserver = FakeResizeObserver as unknown as typeof ResizeObserver;

vi.stubGlobal('setTimeout', vi.fn().mockReturnValue(1));
vi.stubGlobal('clearTimeout', vi.fn());
