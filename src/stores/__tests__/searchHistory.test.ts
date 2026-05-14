import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useSearchHistoryStore } from '../searchHistory';

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem(key: string) {
      return store[key] ?? null;
    },
    setItem(key: string, value: string) {
      store[key] = value;
    },
    removeItem(key: string) {
      delete store[key];
    },
    clear() {
      store = {};
    },
  };
})();

Object.defineProperty(global, 'localStorage', { value: localStorageMock });

describe('useSearchHistoryStore', () => {
  beforeEach(() => {
    localStorageMock.clear();
    setActivePinia(createPinia());
  });

  it('recordSearch: adds new tag with count=1', () => {
    const store = useSearchHistoryStore();
    store.recordSearch('artist_name');
    expect(store.frequencies).toHaveLength(1);
    expect(store.frequencies[0]).toMatchObject({
      tag: 'artist_name',
      count: 1,
    });
    expect(typeof store.frequencies[0].lastSearched).toBe('number');
  });

  it('recordSearch: increments existing tag count', () => {
    const store = useSearchHistoryStore();
    store.recordSearch('artist_name');
    store.recordSearch('artist_name');
    store.recordSearch('artist_name');
    expect(store.frequencies).toHaveLength(1);
    expect(store.frequencies[0].count).toBe(3);
  });

  it('recordSearch: updates lastSearched timestamp', async () => {
    const store = useSearchHistoryStore();
    const before = Date.now();
    store.recordSearch('artist_name');
    const after = Date.now();
    expect(store.frequencies[0].lastSearched).toBeGreaterThanOrEqual(before);
    expect(store.frequencies[0].lastSearched).toBeLessThanOrEqual(after);
  });

  it('recordSearch: sorts by frequency desc', () => {
    const store = useSearchHistoryStore();
    store.recordSearch('tag_a'); // 1
    store.recordSearch('tag_b');
    store.recordSearch('tag_b'); // 2
    store.recordSearch('tag_c');
    store.recordSearch('tag_c');
    store.recordSearch('tag_c'); // 3

    const top = store.getTopTags(3);
    expect(top[0].tag).toBe('tag_c');
    expect(top[1].tag).toBe('tag_b');
    expect(top[2].tag).toBe('tag_a');
  });

  it('getTopTags(limit): returns correct number of tags', () => {
    const store = useSearchHistoryStore();
    for (let i = 0; i < 10; i++) {
      store.recordSearch(`tag_${i}`);
    }
    const top = store.getTopTags(4);
    expect(top).toHaveLength(4);
  });

  it('getTopTags(limit): returns tags sorted by count desc', () => {
    const store = useSearchHistoryStore();
    store.recordSearch('low');
    for (let i = 0; i < 5; i++) store.recordSearch('high');
    for (let i = 0; i < 3; i++) store.recordSearch('medium');

    const top = store.getTopTags(3);
    expect(top[0].count).toBeGreaterThanOrEqual(top[1].count);
    expect(top[1].count).toBeGreaterThanOrEqual(top[2].count);
  });

  it('clearHistory: resets all frequencies', () => {
    const store = useSearchHistoryStore();
    store.recordSearch('tag_a');
    store.recordSearch('tag_b');
    expect(store.frequencies).toHaveLength(2);
    store.clearHistory();
    expect(store.frequencies).toHaveLength(0);
  });

  it('load from localStorage: restores frequencies on init', () => {
    // Pre-populate localStorage as if from a previous session
    localStorageMock.setItem(
      'gelbooru:search-history',
      JSON.stringify([
        { tag: 'artist_name', count: 5, lastSearched: 1700000000000 },
        { tag: 'character_tag', count: 3, lastSearched: 1700000001000 },
      ])
    );
    setActivePinia(createPinia());
    const store = useSearchHistoryStore();
    expect(store.frequencies).toHaveLength(2);
    expect(store.frequencies[0].tag).toBe('artist_name');
    expect(store.frequencies[0].count).toBe(5);
  });

  it('save to localStorage: called after recordSearch', () => {
    const store = useSearchHistoryStore();
    store.recordSearch('artist_name');
    const stored = localStorageMock.getItem('gelbooru:search-history');
    expect(stored).not.toBeNull();
    const parsed = JSON.parse(stored!);
    expect(parsed).toHaveLength(1);
    expect(parsed[0].tag).toBe('artist_name');
  });

  it('Max entries: removes lowest frequency when > 100 entries', () => {
    const store = useSearchHistoryStore();
    // Add 101 tags, each searched once (all count=1)
    for (let i = 0; i < 101; i++) {
      store.recordSearch(`tag_${i}`);
    }
    expect(store.frequencies).toHaveLength(100);
  });

  it('handles empty tag gracefully', () => {
    const store = useSearchHistoryStore();
    store.recordSearch('');
    expect(store.frequencies).toHaveLength(0);
  });

  it('handles invalid localStorage gracefully', () => {
    localStorageMock.setItem('gelbooru:search-history', 'not valid json');
    setActivePinia(createPinia());
    const store = useSearchHistoryStore();
    expect(store.frequencies).toHaveLength(0);
  });
});