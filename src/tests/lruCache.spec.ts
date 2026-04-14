/**
 * LRU Cache unit tests
 */
import { describe, it, expect, beforeEach } from 'vitest';
import { LruCache } from '../utils/lruCache';

describe('LruCache', () => {
  let cache: LruCache<string>;

  beforeEach(() => {
    cache = new LruCache<string>(3);
  });

  it('should store and retrieve values', () => {
    cache.set('a', '1');
    expect(cache.get('a')).toBe('1');
  });

  it('should return undefined for missing keys', () => {
    expect(cache.get('nonexistent')).toBeUndefined();
  });

  it('should evict least recently used when at capacity', () => {
    cache.set('a', '1');
    cache.set('b', '2');
    cache.set('c', '3');
    // Cache is now full: [a, b, c] (c is most recent)

    cache.set('d', '4');
    // 'a' should be evicted

    expect(cache.get('a')).toBeUndefined();
    expect(cache.get('b')).toBe('2');
    expect(cache.get('c')).toBe('3');
    expect(cache.get('d')).toBe('4');
  });

  it('should update existing key and move to most recent', () => {
    cache.set('a', '1');
    cache.set('b', '2');
    cache.set('c', '3');

    cache.set('a', '1-updated');

    // Access order should be: [b, c, a]
    expect(cache.get('b')).toBe('2');
    expect(cache.get('c')).toBe('3');
    expect(cache.get('a')).toBe('1-updated');
  });

  it('should move accessed entry to most recent', () => {
    cache.set('a', '1');
    cache.set('b', '2');
    cache.set('c', '3');

    // Access 'a', making it most recent
    cache.get('a');

    // Add new entry, should evict 'b' (now oldest)
    cache.set('d', '4');

    expect(cache.get('a')).toBe('1');
    expect(cache.get('b')).toBeUndefined();
    expect(cache.get('c')).toBe('3');
    expect(cache.get('d')).toBe('4');
  });

  it('should handle delete correctly', () => {
    cache.set('a', '1');
    cache.set('b', '2');

    cache.delete('a');

    expect(cache.has('a')).toBe(false);
    expect(cache.has('b')).toBe(true);
    expect(cache.size).toBe(1);
  });

  it('should clear all entries', () => {
    cache.set('a', '1');
    cache.set('b', '2');
    cache.set('c', '3');

    cache.clear();

    expect(cache.size).toBe(0);
    expect(cache.get('a')).toBeUndefined();
  });

  it('should report correct size', () => {
    expect(cache.size).toBe(0);

    cache.set('a', '1');
    expect(cache.size).toBe(1);

    cache.set('b', '2');
    expect(cache.size).toBe(2);

    cache.set('c', '3');
    expect(cache.size).toBe(3);

    cache.set('d', '4');
    expect(cache.size).toBe(3);
  });

  it('should throw error for invalid maxSize', () => {
    expect(() => new LruCache(0)).toThrow();
    expect(() => new LruCache(-1)).toThrow();
  });

  it('should work with large cache size', () => {
    const largeCache = new LruCache<number>(1000);
    for (let i = 0; i < 1000; i++) {
      largeCache.set(`key-${i}`, i);
    }
    expect(largeCache.size).toBe(1000);

    // Adding one more should evict the oldest
    largeCache.set('key-1000', 1000);
    expect(largeCache.size).toBe(1000);
    expect(largeCache.get('key-0')).toBeUndefined();
    expect(largeCache.get('key-1000')).toBe(1000);
  });

  it('should correctly report has() for existing and non-existing keys', () => {
    cache.set('a', '1');
    expect(cache.has('a')).toBe(true);
    expect(cache.has('b')).toBe(false);
  });

  it('should handle re-insertion after deletion', () => {
    cache.set('a', '1');
    cache.set('b', '2');
    cache.delete('a');
    cache.set('a', '1-reinserted');

    expect(cache.has('a')).toBe(true);
    expect(cache.get('a')).toBe('1-reinserted');
    expect(cache.size).toBe(2);
  });

  it('should work with default cache size', () => {
    // Default size is 100
    const defaultCache = new LruCache<string>();
    for (let i = 0; i < 100; i++) {
      defaultCache.set(`key-${i}`, `value-${i}`);
    }
    expect(defaultCache.size).toBe(100);
    expect(defaultCache.maxSizeValue).toBe(100);

    // Add one more to trigger eviction
    defaultCache.set('key-100', 'value-100');
    expect(defaultCache.size).toBe(100);
    expect(defaultCache.get('key-0')).toBeUndefined();
  });
});
