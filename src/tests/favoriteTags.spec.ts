/**
 * FavoriteTags store unit tests
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { setActivePinia, createPinia } from 'pinia';
import { useFavoriteTagsStore } from '@/stores/favoriteTags';
import type { FavoriteTagGroup } from '@/types';

vi.mock('@tauri-apps/api/core');

describe('useFavoriteTagsStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  const mockTagGroups: FavoriteTagGroup[] = [
    {
      parent: { id: 1, tag: 'saber', tagType: 'character' },
      children: [
        { id: 2, tag: 'saber_alter', tagType: 'character' },
        { id: 3, tag: 'saber_lancer', tagType: 'character' },
      ],
    },
    {
      parent: { id: 4, tag: 'fate_series', tagType: 'copyright' },
      children: [
        { id: 5, tag: 'fate_stay_night', tagType: 'copyright' },
        { id: 6, tag: 'fate_zero', tagType: 'copyright' },
      ],
    },
    {
      parent: { id: 7, tag: 'solo_artist', tagType: 'artist' },
      children: [],
    },
  ];

  it('should initialize with default values', () => {
    const store = useFavoriteTagsStore();

    expect(store.tags).toEqual([]);
    expect(store.loading).toBe(false);
  });

  describe('loadTags', () => {
    it('should load tags from backend', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(mockTagGroups);

      const store = useFavoriteTagsStore();
      await store.loadTags();

      expect(store.tags).toEqual(mockTagGroups);
      expect(store.loading).toBe(false);
    });

    it('should set loading state during load', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(mockTagGroups);

      const store = useFavoriteTagsStore();
      const loadPromise = store.loadTags();

      expect(store.loading).toBe(true);

      await loadPromise;
      expect(store.loading).toBe(false);
    });

    it('should handle load error gracefully', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('Failed to load'));

      const store = useFavoriteTagsStore();
      await store.loadTags();

      expect(store.tags).toEqual([]);
      expect(store.loading).toBe(false);
    });
  });

  describe('addParentTag', () => {
    it('should add parent tag and reload', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(1) // add_parent_tag returns new id
        .mockResolvedValueOnce(mockTagGroups); // loadTags

      const store = useFavoriteTagsStore();
      await store.addParentTag('new_tag', 'general');

      expect(invoke).toHaveBeenCalledWith('add_parent_tag', {
        tag: 'new_tag',
        tagType: 'general',
      });
      expect(store.tags).toEqual(mockTagGroups);
    });

    it('should use default tagType when not specified', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(1).mockResolvedValueOnce(mockTagGroups);

      const store = useFavoriteTagsStore();
      await store.addParentTag('new_tag');

      expect(invoke).toHaveBeenCalledWith('add_parent_tag', {
        tag: 'new_tag',
        tagType: 'copyright',
      });
    });

    it('should throw error when add fails', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('Add failed'));

      const store = useFavoriteTagsStore();

      await expect(store.addParentTag('new_tag')).rejects.toThrow('Add failed');
    });
  });

  describe('addChildTag', () => {
    it('should add child tag to parent and reload', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(8) // add_child_tag returns new id
        .mockResolvedValueOnce(mockTagGroups); // loadTags

      const store = useFavoriteTagsStore();
      await store.addChildTag('child_tag', 'general', 1);

      expect(invoke).toHaveBeenCalledWith('add_child_tag', {
        tag: 'child_tag',
        tagType: 'general',
        parentId: 1,
      });
    });

    it('should throw error when add fails', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('Add failed'));

      const store = useFavoriteTagsStore();

      await expect(store.addChildTag('child', 'general', 1)).rejects.toThrow('Add failed');
    });
  });

  describe('removeTag', () => {
    it('should remove tag and reload', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(undefined).mockResolvedValueOnce(mockTagGroups);

      const store = useFavoriteTagsStore();
      await store.removeTag(1);

      expect(invoke).toHaveBeenCalledWith('remove_favorite_tag', { id: 1 });
    });

    it('should throw error when remove fails', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('Remove failed'));

      const store = useFavoriteTagsStore();

      await expect(store.removeTag(1)).rejects.toThrow('Remove failed');
    });
  });

  describe('isTagFavorited', () => {
    it('should return true when tag is favorited', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(true);

      const store = useFavoriteTagsStore();
      const result = await store.isTagFavorited('saber');

      expect(result).toBe(true);
    });

    it('should return false when tag is not favorited', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(false);

      const store = useFavoriteTagsStore();
      const result = await store.isTagFavorited('unknown');

      expect(result).toBe(false);
    });

    it('should return false on error', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('Check failed'));

      const store = useFavoriteTagsStore();
      const result = await store.isTagFavorited('saber');

      expect(result).toBe(false);
    });
  });

  describe('findTagGroup', () => {
    beforeEach(async () => {
      vi.mocked(invoke).mockResolvedValueOnce(mockTagGroups);
      const store = useFavoriteTagsStore();
      await store.loadTags();
    });

    it('should find group by parent tag', () => {
      const store = useFavoriteTagsStore();
      const group = store.findTagGroup('saber');

      expect(group).toBeDefined();
      expect(group?.parent.tag).toBe('saber');
    });

    it('should find group by child tag', () => {
      const store = useFavoriteTagsStore();
      const group = store.findTagGroup('saber_alter');

      expect(group).toBeDefined();
      expect(group?.parent.tag).toBe('saber');
    });

    it('should return undefined for unknown tag', () => {
      const store = useFavoriteTagsStore();
      const group = store.findTagGroup('unknown_tag');

      expect(group).toBeUndefined();
    });

    it('should find group with empty children', () => {
      const store = useFavoriteTagsStore();
      const group = store.findTagGroup('solo_artist');

      expect(group).toBeDefined();
      expect(group?.children).toEqual([]);
    });
  });
});
