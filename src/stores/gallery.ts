import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { GelbooruPost, GelbooruTag } from '@/types'

export const useGalleryStore = defineStore('gallery', () => {
  const posts = ref<GelbooruPost[]>([])
  const tags = ref<GelbooruTag[]>([])
  const currentPage = ref(1)
  const totalPages = ref(1)
  const searchTags = ref<string[]>([])
  const loading = ref(false)
  const totalPosts = ref(0)
  
  function setPosts(newPosts: GelbooruPost[]) {
    posts.value = newPosts
  }
  
  function appendPosts(newPosts: GelbooruPost[]) {
    posts.value.push(...newPosts)
  }
  
  function setTags(newTags: GelbooruTag[]) {
    tags.value = newTags
  }
  
  function setTotalPages(pages: number) {
    totalPages.value = pages
  }
  
  function setSearchTags(tags: string[]) {
    searchTags.value = tags
    currentPage.value = 1
    posts.value = []
  }
  
  function nextPage() {
    currentPage.value++
  }
  
  function setLoading(value: boolean) {
    loading.value = value
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
    setLoading
  }
})
