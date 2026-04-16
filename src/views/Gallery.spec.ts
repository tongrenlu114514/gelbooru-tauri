import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import { nextTick } from 'vue';
import { LruCache } from '../utils/lruCache';

// ---------------------------------------------------------------------------
// vi.hoisted — ensures variables are available during vi.mock hoisting.
// These must be declared before any vi.mock calls.
// ---------------------------------------------------------------------------
const { observeSpy, unobserveSpy, disconnectSpy, takeRecordsSpy, fakeIntersectionObserver } = vi.hoisted(() => {
  const observe = vi.fn();
  const unobserve = vi.fn();
  const disconnect = vi.fn();
  const takeRecords = vi.fn(() => []);

  let observerCallback: ((entries: IntersectionObserverEntry[]) => void) | null = null;

  // fakeIntersectionObserver must be a vi.fn() spy so .mock.calls is accessible
  const fakeIO = vi.fn(function (
    this: unknown,
    callback: (entries: IntersectionObserverEntry[]) => void
  ) {
    observerCallback = callback;
    return {
      observe,
      unobserve,
      disconnect,
      takeRecords,
    };
  });

  return { observeSpy: observe, unobserveSpy: unobserve, disconnectSpy: disconnect, takeRecordsSpy: takeRecords, fakeIntersectionObserver: fakeIO };
});

// observerCallback is declared inside vi.hoisted scope above; module-level reference not needed

// ---------------------------------------------------------------------------
// @tauri-apps/api/core mock — setup.ts handles the mock; reference its exports
// ---------------------------------------------------------------------------
vi.mock('@tauri-apps/api/core', async (importOriginal) => {
  const actual = await importOriginal<Record<string, unknown>>();
  return {
    ...actual,
    invoke: vi.fn(),
    convertFileSrc: vi.fn((path: string) => `mock://asset/${path}`),
  };
});

// ---------------------------------------------------------------------------
// naive-ui mock — provides stubs for all components + mock composables
// ---------------------------------------------------------------------------
vi.mock('naive-ui', async (importOriginal) => {
  const actual = await importOriginal<Record<string, unknown>>();
  return {
    ...actual,
    useMessage: () => ({ info: vi.fn(), success: vi.fn(), error: vi.fn(), warning: vi.fn() }),
    useDialog: () => ({ warning: vi.fn(), info: vi.fn(), error: vi.fn(), success: vi.fn() }),
    NMessageProvider: ({ children }: { children?: unknown }) => children as ReturnType<typeof importOriginal>,
    NDialogProvider: ({ children }: { children?: unknown }) => children as ReturnType<typeof importOriginal>,
  };
});

// ---------------------------------------------------------------------------
// Component under test — imports AFTER mocks are established
// ---------------------------------------------------------------------------
import Gallery from './Gallery.vue';
import { invoke, convertFileSrc } from '@tauri-apps/api/core';

const mockInvoke = invoke as ReturnType<typeof vi.fn>;
const mockConvertFileSrc = convertFileSrc as ReturnType<typeof vi.fn>;

// Set up IntersectionObserver on window AFTER mocks are set up but BEFORE mount
Object.defineProperty(window, 'IntersectionObserver', {
  value: fakeIntersectionObserver,
  writable: true,
  configurable: true,
});

const MOCK_TREE = [
  { key: 'dir1', label: 'dir1', path: '/base/dir1', isLeaf: false, imageCount: 1, children: [] },
];

const BASE_OPTS = {
  global: {
    stubs: {
      'n-space': true,
      'n-button': true,
      'n-icon': true,
      'n-layout': true,
      'n-layout-sider': true,
      'n-layout-content': true,
      'n-tree': true,
      'n-empty': true,
      'n-spin': true,
      'n-modal': true,
      'n-text': true,
    },
  },
};

describe('Gallery.vue — IntersectionObserver lazy loading', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockReset();
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_directory_tree') return Promise.resolve(MOCK_TREE);
      if (cmd === 'get_local_image_base64') return Promise.resolve('mock-base64-data');
      return Promise.resolve(undefined);
    });
    mockConvertFileSrc.mockImplementation((path: string) => `mock://asset/${path}`);
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  // -------------------------------------------------------------------------
  // Test 1: observerRef is created on mount with correct options
  // -------------------------------------------------------------------------
  it('creates IntersectionObserver on mount with rootMargin 200px and threshold 0.01', async () => {
    const wrapper = mount(Gallery, BASE_OPTS);
    await flushPromises();

    expect(fakeIntersectionObserver).toHaveBeenCalledTimes(1);
    const observerCall = fakeIntersectionObserver.mock.calls[0];
    const options = observerCall[1];
    expect(options).toMatchObject({
      root: null,
      rootMargin: '200px',
      threshold: 0.01,
    });

    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 2: loadVisibleImages calls observer.observe on image cards
  // -------------------------------------------------------------------------
  it('calls observer.observe on all image cards when loadVisibleImages runs', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_directory_tree') return Promise.resolve(MOCK_TREE);
      if (cmd === 'get_local_image_base64') return Promise.resolve('base64data');
      return Promise.resolve(undefined);
    });

    const wrapper = mount(Gallery, BASE_OPTS);
    await flushPromises();

    // Build a DOM that matches what Gallery.vue's loadVisibleImages expects:
    // a .content-grid div containing [data-image-path] cards.
    const grid = document.createElement('div');
    grid.className = 'content-grid';

    const card1 = document.createElement('div');
    card1.dataset.imagePath = '/base/dir1/img1.jpg';
    const card2 = document.createElement('div');
    card2.dataset.imagePath = '/base/dir1/img2.jpg';

    grid.appendChild(card1);
    grid.appendChild(card2);
    document.body.appendChild(grid);

    await nextTick();

    // Call loadVisibleImages via the exposed ref (defineExpose in Gallery.vue)
    const vm = wrapper.vm as Record<string, unknown>;
    const fn = vm.loadVisibleImages as (() => void) | undefined;
    expect(fn).toBeDefined();
    fn!();

    // Verify observe was called for each card with data-image-path
    expect(observeSpy).toHaveBeenCalledTimes(2);
    expect(observeSpy).toHaveBeenCalledWith(card1);
    expect(observeSpy).toHaveBeenCalledWith(card2);

    document.body.removeChild(grid);
    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 3: When IntersectionObserver fires with isIntersecting=true, loadImageBase64 is called
  // -------------------------------------------------------------------------
  it('loads base64 when image element enters the viewport', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_directory_tree') return Promise.resolve(MOCK_TREE);
      if (cmd === 'get_local_image_base64') return Promise.resolve('base64-encoded-image');
      return Promise.resolve(undefined);
    });

    const wrapper = mount(Gallery, BASE_OPTS);
    await flushPromises();

    // Simulate the observer callback with an intersecting entry
    const fakeEntry = {
      target: { dataset: { imagePath: '/base/dir1/img1.jpg' } },
      isIntersecting: true,
      intersectionRatio: 1,
      boundingClientRect: {} as DOMRectReadOnly,
      intersectionRect: {} as DOMRectReadOnly,
      rootBounds: null,
      time: Date.now(),
    } as unknown as IntersectionObserverEntry;

    const callback = fakeIntersectionObserver.mock.calls[0][0];
    callback([fakeEntry]);
    await flushPromises();

    expect(mockInvoke).toHaveBeenCalledWith('get_local_image_base64', {
      path: '/base/dir1/img1.jpg',
    });

    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 4: When IntersectionObserver fires with isIntersecting=false, image is NOT loaded
  // -------------------------------------------------------------------------
  it('does NOT load image when element leaves the viewport', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_directory_tree') return Promise.resolve(MOCK_TREE);
      if (cmd === 'get_local_image_base64') return Promise.resolve('base64-encoded-image');
      return Promise.resolve(undefined);
    });

    const wrapper = mount(Gallery, BASE_OPTS);
    await flushPromises();

    const fakeEntry = {
      target: { dataset: { imagePath: '/base/dir1/img1.jpg' } },
      isIntersecting: false,
      intersectionRatio: 0,
      boundingClientRect: {} as DOMRectReadOnly,
      intersectionRect: {} as DOMRectReadOnly,
      rootBounds: null,
      time: Date.now(),
    } as unknown as IntersectionObserverEntry;

    const callback = fakeIntersectionObserver.mock.calls[0][0];
    callback([fakeEntry]);
    await flushPromises();

    expect(mockInvoke).not.toHaveBeenCalledWith('get_local_image_base64', {
      path: '/base/dir1/img1.jpg',
    });

    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 5: observerRef.disconnect() is called on component unmount
  // -------------------------------------------------------------------------
  it('disconnects IntersectionObserver on component unmount', async () => {
    const wrapper = mount(Gallery, BASE_OPTS);
    await flushPromises();

    const observer = fakeIntersectionObserver.mock.results[0].value as {
      disconnect: ReturnType<typeof vi.fn>;
    };

    wrapper.unmount();

    expect(observer.disconnect).toHaveBeenCalledTimes(1);
  });

  // -------------------------------------------------------------------------
  // Test 6: Duplicate images are not re-loaded into cache
  // -------------------------------------------------------------------------
  it('does not reload an image already in the LRU cache', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_directory_tree') return Promise.resolve(MOCK_TREE);
      if (cmd === 'get_local_image_base64') return Promise.resolve('base64-encoded-image');
      return Promise.resolve(undefined);
    });

    const wrapper = mount(Gallery, BASE_OPTS);
    await flushPromises();

    const fakeEntry = {
      target: { dataset: { imagePath: '/base/dir1/img1.jpg' } },
      isIntersecting: true,
      intersectionRatio: 1,
      boundingClientRect: {} as DOMRectReadOnly,
      intersectionRect: {} as DOMRectReadOnly,
      rootBounds: null,
      time: Date.now(),
    } as unknown as IntersectionObserverEntry;

    const callback = fakeIntersectionObserver.mock.calls[0][0];
    // First visibility — should call invoke
    callback([fakeEntry]);
    await flushPromises();
    expect(mockInvoke).toHaveBeenCalledTimes(1);

    // Second visibility — cache hit, should NOT call invoke
    mockInvoke.mockClear();
    callback([fakeEntry]);
    await flushPromises();

    expect(mockInvoke).not.toHaveBeenCalled();

    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 7: refresh() disconnects the observer before clearing state
  // -------------------------------------------------------------------------
  it('refresh disconnects observer and clears images', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'get_directory_tree') return Promise.resolve(MOCK_TREE);
      return Promise.resolve(undefined);
    });

    const wrapper = mount(Gallery, BASE_OPTS);
    await flushPromises();

    const observer = fakeIntersectionObserver.mock.results[0].value as {
      disconnect: ReturnType<typeof vi.fn>;
    };

    // Trigger refresh button click
    const buttons = wrapper.findAll('button');
    const refreshBtn = buttons.find((btn) => btn.text().includes('刷新'));
    if (refreshBtn) {
      await refreshBtn.trigger('click');
      await flushPromises();
    }

    expect(observer.disconnect).toHaveBeenCalled();

    wrapper.unmount();
  });

  // -------------------------------------------------------------------------
  // Test 8: LRU cache evicts oldest entry when at capacity
  // -------------------------------------------------------------------------
  it('LRU cache evicts oldest entry when at capacity', () => {
    const cache = new LruCache<string>(3);

    cache.set('key1', 'value1');
    cache.set('key2', 'value2');
    cache.set('key3', 'value3');

    // Cache is at capacity
    expect(cache.size).toBe(3);

    // Adding a 4th entry should evict the oldest (key1)
    cache.set('key4', 'value4');

    expect(cache.size).toBe(3);
    expect(cache.has('key1')).toBe(false); // evicted
    expect(cache.has('key2')).toBe(true);
    expect(cache.has('key3')).toBe(true);
    expect(cache.has('key4')).toBe(true);

    // Accessing key2 should move it to most recent
    cache.get('key2');
    cache.set('key5', 'value5');

    // key3 should now be evicted instead of key2
    expect(cache.has('key3')).toBe(false);
    expect(cache.has('key2')).toBe(true);
    expect(cache.has('key4')).toBe(true);
    expect(cache.has('key5')).toBe(true);
  });
});
