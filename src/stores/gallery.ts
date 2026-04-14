import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { GelbooruPost, GelbooruTag } from '@/types';

// 页面状态（用于切换页面后恢复）
interface PageState {
  selectedTags: string[];
  selectedRating: string;
  currentPage: number;
  posts: GelbooruPost[];
  tags: GelbooruTag[];
  totalPages: number;
  searchTags: string[];
}

export const useGalleryStore = defineStore('gallery', () => {
  const posts = ref<GelbooruPost[]>([]);
  const tags = ref<GelbooruTag[]>([]);
  const currentPage = ref(1);
  const totalPages = ref(1);
  const searchTags = ref<string[]>([]);
  const loading = ref(false);
  const totalPosts = ref(0);

  // 页面状态
  const pageState = ref<PageState | null>(null);

  function setPosts(newPosts: GelbooruPost[]) {
    posts.value = newPosts;
  }

  function appendPosts(newPosts: GelbooruPost[]) {
    posts.value.push(...newPosts);
  }

  function setTags(newTags: GelbooruTag[]) {
    tags.value = newTags;
  }

  function setTotalPages(pages: number) {
    totalPages.value = pages;
  }

  function setSearchTags(tags: string[]) {
    searchTags.value = tags;
  }

  function nextPage() {
    currentPage.value++;
  }

  function setLoading(value: boolean) {
    loading.value = value;
  }

  // 保存页面状态（离开页面前调用）
  function savePageState(selectedTags: string[], selectedRating: string) {
    pageState.value = {
      selectedTags,
      selectedRating,
      currentPage: currentPage.value,
      posts: posts.value,
      tags: tags.value,
      totalPages: totalPages.value,
      searchTags: searchTags.value,
    };
    console.debug('[PageState] Saved');
  }

  // 恢复页面状态（返回页面时调用）
  function restorePageState(): PageState | null {
    if (pageState.value) {
      console.debug('[PageState] Restored');
      return pageState.value;
    }
    return null;
  }

  // 清除页面状态
  function clearPageState() {
    pageState.value = null;
    console.debug('[PageState] Cleared');
  }

  return {
    posts,
    tags,
    currentPage,
    totalPages,
    searchTags,
    loading,
    totalPosts,
    setPosts,
    appendPosts,
    setTags,
    setTotalPages,
    setSearchTags,
    nextPage,
    setLoading,
    savePageState,
    restorePageState,
    clearPageState,
  };
});
