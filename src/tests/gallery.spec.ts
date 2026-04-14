/**
 * Gallery store unit tests
 */
import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useGalleryStore } from '@/stores/gallery';
import type { GelbooruPost, GelbooruTag } from '@/types';

describe('useGalleryStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  const mockPosts: GelbooruPost[] = [
    {
      id: 1,
      url: 'https://example.com/1',
      title: 'Post 1',
      tagList: [
        { text: 'blue eyes', tagType: 'general', count: 100 },
        { text: 'saber', tagType: 'character', count: 50 },
      ],
      statistics: {
        size: '1920x1080',
        rating: 'safe',
        posted: '2024-01-01',
        source: '',
        score: 100,
        image: 'https://example.com/image1.jpg',
        sample: 'https://example.com/thumb1.jpg',
      },
    },
    {
      id: 2,
      url: 'https://example.com/2',
      title: 'Post 2',
      tagList: [{ text: 'red eyes', tagType: 'general', count: 80 }],
      statistics: {
        size: '1280x720',
        rating: 'questionable',
        posted: '2024-01-02',
        source: '',
        score: 50,
        image: 'https://example.com/image2.jpg',
        sample: 'https://example.com/thumb2.jpg',
      },
    },
  ];

  const mockTags: GelbooruTag[] = [
    { text: 'blue eyes', tagType: 'general', count: 100 },
    { text: 'saber', tagType: 'character', count: 50 },
  ];

  it('should initialize with default values', () => {
    const store = useGalleryStore();

    expect(store.posts).toEqual([]);
    expect(store.tags).toEqual([]);
    expect(store.currentPage).toBe(1);
    expect(store.totalPages).toBe(1);
    expect(store.searchTags).toEqual([]);
    expect(store.loading).toBe(false);
    expect(store.totalPosts).toBe(0);
  });

  describe('setPosts', () => {
    it('should set posts array', () => {
      const store = useGalleryStore();
      store.setPosts(mockPosts);

      expect(store.posts).toEqual(mockPosts);
    });

    it('should replace existing posts', () => {
      const store = useGalleryStore();
      store.setPosts(mockPosts);
      store.setPosts([mockPosts[0]]);

      expect(store.posts).toHaveLength(1);
      expect(store.posts[0].id).toBe(1);
    });
  });

  describe('appendPosts', () => {
    it('should append posts to existing array', () => {
      const store = useGalleryStore();
      store.setPosts([mockPosts[0]]);
      store.appendPosts([mockPosts[1]]);

      expect(store.posts).toHaveLength(2);
      expect(store.posts[0].id).toBe(1);
      expect(store.posts[1].id).toBe(2);
    });
  });

  describe('setTags', () => {
    it('should set tags array', () => {
      const store = useGalleryStore();
      store.setTags(mockTags);

      expect(store.tags).toEqual(mockTags);
    });
  });

  describe('setTotalPages', () => {
    it('should set total pages', () => {
      const store = useGalleryStore();
      store.setTotalPages(10);

      expect(store.totalPages).toBe(10);
    });
  });

  describe('setSearchTags', () => {
    it('should set search tags', () => {
      const store = useGalleryStore();
      store.setSearchTags(['blue_eyes', 'saber']);

      expect(store.searchTags).toEqual(['blue_eyes', 'saber']);
    });
  });

  describe('nextPage', () => {
    it('should increment current page', () => {
      const store = useGalleryStore();
      expect(store.currentPage).toBe(1);

      store.nextPage();
      expect(store.currentPage).toBe(2);

      store.nextPage();
      expect(store.currentPage).toBe(3);
    });
  });

  describe('setLoading', () => {
    it('should set loading state', () => {
      const store = useGalleryStore();
      expect(store.loading).toBe(false);

      store.setLoading(true);
      expect(store.loading).toBe(true);

      store.setLoading(false);
      expect(store.loading).toBe(false);
    });
  });

  describe('pageState', () => {
    it('should save and restore page state', () => {
      const store = useGalleryStore();

      // Set up state
      store.setPosts(mockPosts);
      store.setTags(mockTags);
      store.setSearchTags(['blue_eyes']);
      store.setTotalPages(10);

      // Save state
      store.savePageState(['blue_eyes'], 'safe');

      // Modify current state
      store.setPosts([]);
      store.setTags([]);
      store.setSearchTags([]);
      store.currentPage = 1;

      // Restore state
      const restored = store.restorePageState();

      expect(restored).toBeDefined();
      expect(restored?.posts).toEqual(mockPosts);
      expect(restored?.tags).toEqual(mockTags);
      expect(restored?.searchTags).toEqual(['blue_eyes']);
      expect(restored?.totalPages).toBe(10);
      expect(restored?.selectedTags).toEqual(['blue_eyes']);
      expect(restored?.selectedRating).toBe('safe');
      expect(restored?.currentPage).toBe(1);
    });

    it('should return null when no state saved', () => {
      const store = useGalleryStore();
      const restored = store.restorePageState();

      expect(restored).toBeNull();
    });

    it('should clear page state', () => {
      const store = useGalleryStore();
      store.savePageState(['tag'], 'safe');

      store.clearPageState();

      const restored = store.restorePageState();
      expect(restored).toBeNull();
    });
  });
});
