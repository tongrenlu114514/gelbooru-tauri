import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { GelbooruPost, GelbooruTag } from '@/types'

interface CachedResult {
  posts: GelbooruPost[]
  tags: GelbooruTag[]
  totalPages: number
  searchTags: string[]  // 搜索条件
  timestamp: number
}

function createCacheKey(tags: string[], page: number): string {
  const sortedTags = [...tags].sort().join(',')
  return `${sortedTags}:${page}`
}

export const useGalleryStore = defineStore('gallery', () => {
  const posts = ref<GelbooruPost[]>([])
  const tags = ref<GelbooruTag[]>([])
  const currentPage = ref(1)
  const totalPages = ref(1)
  const searchTags = ref<string[]>([])
  const loading = ref(false)
  const totalPosts = ref(0)
  
  // 缓存：key = "tags:page", value = CachedResult
  const cache = new Map<string, CachedResult>()
  const CACHE_EXPIRE_TIME = 10 * 60 * 1000 // 10分钟过期
  
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
  
  // 获取缓存
  function getCache(tags: string[], page: number): CachedResult | null {
    const key = createCacheKey(tags, page)
    const cached = cache.get(key)
    
    if (cached) {
      // 检查是否过期
      if (Date.now() - cached.timestamp < CACHE_EXPIRE_TIME) {
        console.log('[Cache] Hit:', key)
        return cached
      } else {
        // 过期则删除
        cache.delete(key)
        console.log('[Cache] Expired:', key)
      }
    }
    
    return null
  }
  
  // 设置缓存
  function setCache(tags: string[], page: number, result: Omit<CachedResult, 'timestamp'>) {
    const key = createCacheKey(tags, page)
    cache.set(key, {
      ...result,
      timestamp: Date.now()
    })
    console.log('[Cache] Set:', key)
  }
  
  // 清除缓存
  function clearCache() {
    cache.clear()
    console.log('[Cache] Cleared')
  }
  
  // 清理过期缓存
  function cleanupExpiredCache() {
    const now = Date.now()
    for (const [key, value] of cache.entries()) {
      if (now - value.timestamp >= CACHE_EXPIRE_TIME) {
        cache.delete(key)
      }
    }
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
    getCache,
    setCache,
    clearCache,
    cleanupExpiredCache
  }
})
