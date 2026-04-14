import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { FavoriteTagGroup } from '@/types';

export const useFavoriteTagsStore = defineStore('favoriteTags', () => {
  const tags = ref<FavoriteTagGroup[]>([]);
  const loading = ref(false);

  async function loadTags() {
    loading.value = true;
    try {
      const result = await invoke<FavoriteTagGroup[]>('get_favorite_tags');
      tags.value = result;
    } catch (error) {
      console.error('Failed to load favorite tags:', error);
    } finally {
      loading.value = false;
    }
  }

  async function addParentTag(tag: string, tagType: string = 'copyright') {
    try {
      await invoke<number>('add_parent_tag', { tag, tagType });
      await loadTags();
    } catch (error) {
      console.error('Failed to add parent tag:', error);
      throw error;
    }
  }

  async function addChildTag(tag: string, tagType: string, parentId: number) {
    try {
      await invoke<number>('add_child_tag', { tag, tagType, parentId });
      await loadTags();
    } catch (error) {
      console.error('Failed to add child tag:', error);
      throw error;
    }
  }

  async function removeTag(id: number) {
    try {
      await invoke('remove_favorite_tag', { id });
      await loadTags();
    } catch (error) {
      console.error('Failed to remove tag:', error);
      throw error;
    }
  }

  async function isTagFavorited(tag: string): Promise<boolean> {
    try {
      return await invoke<boolean>('is_tag_favorited', { tag });
    } catch (error) {
      console.error('Failed to check if tag is favorited:', error);
      return false;
    }
  }

  // 查找tag所在的组
  function findTagGroup(tagText: string): FavoriteTagGroup | undefined {
    return tags.value.find(
      (group) =>
        group.parent.tag === tagText || group.children.some((child) => child.tag === tagText)
    );
  }

  return {
    tags,
    loading,
    loadTags,
    addParentTag,
    addChildTag,
    removeTag,
    isTagFavorited,
    findTagGroup,
  };
});
