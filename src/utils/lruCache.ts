/**
 * LRU (Least Recently Used) Cache implementation
 * Automatically evicts least recently used entries when capacity is reached
 */
export class LruCache<T> {
  private cache: Map<string, T>;
  private readonly maxSize: number;

  constructor(maxSize: number = 100) {
    if (maxSize <= 0) {
      throw new Error('LruCache maxSize must be positive');
    }
    this.maxSize = maxSize;
    this.cache = new Map();
  }

  /**
   * Get a value from the cache
   * Moves the accessed entry to the most recently used position
   */
  get(key: string): T | undefined {
    if (!this.cache.has(key)) {
      return undefined;
    }
    // Move to end (most recently used) by deleting and re-inserting
    const value = this.cache.get(key)!;
    this.cache.delete(key);
    this.cache.set(key, value);
    return value;
  }

  /**
   * Check if cache has a key without updating access order
   */
  has(key: string): boolean {
    return this.cache.has(key);
  }

  /**
   * Set a value in the cache
   * Evicts least recently used entry if at capacity
   */
  set(key: string, value: T): void {
    // If key exists, delete it first (will be re-inserted at end)
    if (this.cache.has(key)) {
      this.cache.delete(key);
    }
    // Evict oldest entry if at capacity
    if (this.cache.size >= this.maxSize) {
      const oldestKey = this.cache.keys().next().value;
      if (oldestKey !== undefined) {
        this.cache.delete(oldestKey);
      }
    }
    // Insert at end (most recently used)
    this.cache.set(key, value);
  }

  /**
   * Remove a specific entry from the cache
   */
  delete(key: string): boolean {
    return this.cache.delete(key);
  }

  /**
   * Clear all entries from the cache
   */
  clear(): void {
    this.cache.clear();
  }

  /**
   * Get current number of entries in cache
   */
  get size(): number {
    return this.cache.size;
  }

  /**
   * Get the maximum capacity of the cache
   */
  get maxSizeValue(): number {
    return this.maxSize;
  }
}

// Default cache instance for image base64 data
const DEFAULT_IMAGE_CACHE_SIZE = 100;
const imageBase64Cache = new LruCache<string>(DEFAULT_IMAGE_CACHE_SIZE);

export { DEFAULT_IMAGE_CACHE_SIZE, imageBase64Cache };
