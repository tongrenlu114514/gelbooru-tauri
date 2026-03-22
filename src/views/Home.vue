<script setup lang="ts">
import { ref, onMounted, watch, computed, nextTick } from 'vue'
import {
  NInput,
  NButton,
  NSpace,
  NTag,
  NSpin,
  NEmpty,
  NSelect,
  NPagination,
  NIcon,
  useMessage
} from 'naive-ui'
import { ChevronBack, ChevronForward, Close } from '@vicons/ionicons5'
import { useGalleryStore } from '@/stores/gallery'
import { useDownloadStore } from '@/stores/download'
import { invoke } from '@tauri-apps/api/core'
import type { GelbooruPost, GelbooruTag } from '@/types'

const galleryStore = useGalleryStore()
const downloadStore = useDownloadStore()
const message = useMessage()

const searchInput = ref('')
const selectedTags = ref<string[]>([])
const loading = ref(false)

// 预览相关状态
const showPreview = ref(false)
const previewLoading = ref(false)
const currentPreviewIndex = ref(0)
const previewImageUrl = ref('')
const previewPost = ref<GelbooruPost | null>(null)

const currentPost = computed(() => previewPost.value || galleryStore.posts[currentPreviewIndex.value])

// Tag 分组配置
const tagTypeConfig: Record<string, { label: string; color: string }> = {
  artist: { label: '艺术家', color: '#4caf50' },
  character: { label: '角色', color: '#e91e63' },
  copyright: { label: '作品', color: '#9c27b0' },
  general: { label: '标签', color: '#2196f3' },
  metadata: { label: '元数据', color: '#607d8b' }
}

// 对 tag 进行分组
const groupedTags = computed(() => {
  if (!previewPost.value?.tagList) return []
  
  const groups: Record<string, { type: string; label: string; color: string; tags: GelbooruTag[] }> = {}
  
  for (const tag of previewPost.value.tagList) {
    const type = tag.tagType.toLowerCase()
    const config = tagTypeConfig[type] || tagTypeConfig.general
    
    if (!groups[type]) {
      groups[type] = {
        type,
        label: config.label,
        color: config.color,
        tags: []
      }
    }
    groups[type].tags.push(tag)
  }
  
  // 按优先级排序：artist > character > copyright > general
  const order = ['artist', 'character', 'copyright', 'general', 'metadata']
  return Object.values(groups).sort((a, b) => {
    return order.indexOf(a.type) - order.indexOf(b.type)
  })
})

// 格式化数量显示
function formatCount(count: number): string {
  if (count >= 1000000) return (count / 1000000).toFixed(1) + 'M'
  if (count >= 1000) return (count / 1000).toFixed(1) + 'K'
  return count.toString()
}

// 点击 tag 进行搜索（空格替换为下划线）
function handleTagClick(tagText: string) {
  const searchTag = tagText.replace(/\s+/g, '_')
  addTag(searchTag)
  closePreview()
  searchPosts(true)
}

const ratingOptions = [
  { label: '全部', value: '' },
  { label: 'Safe', value: 'rating:safe' },
  { label: 'Questionable', value: 'rating:questionable' },
  { label: 'Explicit', value: 'rating:explicit' }
]
const selectedRating = ref('')

async function searchPosts(resetPage = false, forceRefresh = false) {
  // 先把输入框的内容加入标签
  if (searchInput.value.trim()) {
    addTag(searchInput.value.trim())
    searchInput.value = ''
  }
  
  // 重置页数
  if (resetPage) {
    galleryStore.currentPage = 1
  }
  
  // 构建搜索标签
  const tags = [...selectedTags.value]
  if (selectedRating.value) {
    tags.push(selectedRating.value)
  }
  
  // 非强制刷新时先检查缓存
  if (!forceRefresh) {
    const cached = galleryStore.getCache(tags, galleryStore.currentPage)
    if (cached) {
      galleryStore.setPosts(cached.posts)
      galleryStore.setTags(cached.tags)
      galleryStore.setTotalPages(cached.totalPages)
      scrollToTop()
      return
    }
  }
  
  loading.value = true
  try {
    const result = await invoke<{ postList: GelbooruPost[], tagList: GelbooruTag[], totalPages: number }>('search_posts', {
      tags: tags,
      page: galleryStore.currentPage
    })
    
    galleryStore.setPosts(result.postList)
    galleryStore.setTags(result.tagList)
    galleryStore.setTotalPages(result.totalPages)
    
    // 缓存结果
    galleryStore.setCache(tags, galleryStore.currentPage, {
      posts: result.postList,
      tags: result.tagList,
      totalPages: result.totalPages
    })
  } catch (error) {
    console.error('Search failed:', error)
  } finally {
    loading.value = false
    // 滚动到顶部
    scrollToTop()
  }
}

// 滚动到顶部
function scrollToTop() {
  nextTick(() => {
    // 查找所有可能的滚动容器
    const containers = [
      document.querySelector('.n-layout-content'),
      document.querySelector('.n-layout-scroll-container'),
      document.querySelector('.n-scrollbar-container'),
      document.documentElement,
      document.body
    ]
    
    for (const container of containers) {
      if (container && 'scrollTop' in container) {
        (container as HTMLElement).scrollTop = 0
      }
    }
    
    // 最后尝试 scrollIntoView
    const firstElement = document.querySelector('.home-view > *:first-child')
    if (firstElement) {
      firstElement.scrollIntoView({ behavior: 'smooth', block: 'start' })
    }
  })
}

async function openPreview(index: number) {
  currentPreviewIndex.value = index
  previewImageUrl.value = ''
  previewPost.value = null
  previewLoading.value = true
  showPreview.value = true  // 先显示弹窗
  
  try {
    const post = galleryStore.posts[index]
    console.log('[DEBUG] Opening preview for post:', post.id)
    
    const detail = await invoke<GelbooruPost>('get_post_detail', { id: post.id })
    console.log('[DEBUG] Got post detail:', detail)
    console.log('[DEBUG] Sample URL:', detail.statistics.sample)
    console.log('[DEBUG] Image URL:', detail.statistics.image)
    
    previewPost.value = detail
    
    // 使用 sample URL 预览（更快加载）
    const previewUrl = detail.statistics.sample || detail.statistics.image
    const base64Url = await invoke<string>('get_image_base64', { url: previewUrl })
    console.log('[DEBUG] Got base64 image, length:', base64Url.length)
    previewImageUrl.value = base64Url
  } catch (error) {
    console.error('[ERROR] Failed to load preview:', error)
    // 使用缩略图作为后备
    previewImageUrl.value = galleryStore.posts[index].thumbnail || ''
  } finally {
    previewLoading.value = false
  }
}

function closePreview() {
  showPreview.value = false
}

async function prevImage() {
  if (currentPreviewIndex.value > 0) {
    await openPreview(currentPreviewIndex.value - 1)
  }
}

async function nextImage() {
  if (currentPreviewIndex.value < galleryStore.posts.length - 1) {
    await openPreview(currentPreviewIndex.value + 1)
  }
}

async function downloadPost(post: GelbooruPost) {
  // 先获取详情以获取原图 URL 和完整标签
  let imageUrl = post.statistics.image;
  let tags = post.tagList || [];
  let posted = post.statistics.posted || '';
  let rating = post.statistics.rating || '';
  
  if (!imageUrl || tags.length === 0) {
    try {
      const detail = await invoke<GelbooruPost>('get_post_detail', { id: post.id });
      imageUrl = detail.statistics.image;
      tags = detail.tagList || [];
      posted = detail.statistics.posted || '';
      rating = detail.statistics.rating || '';
    } catch (error) {
      console.error('Failed to get post detail for download:', error);
      message.error('获取图片详情失败');
      return;
    }
  }
  
  if (!imageUrl) {
    message.error('无法获取图片地址');
    return;
  }
  
  try {
    await downloadStore.addTask({
      postId: post.id,
      imageUrl: imageUrl,
      posted: posted,
      rating: rating,
      tags: tags
    });
    message.success('已添加到下载队列');
  } catch (error) {
    console.error('Failed to add download task:', error);
    message.error('添加下载任务失败');
  }
}

async function downloadCurrentPost() {
  if (previewPost.value) {
    const imageUrl = previewPost.value.statistics.image;
    if (!imageUrl) {
      message.error('无法获取图片地址');
      return;
    }
    
    try {
      await downloadStore.addTask({
        postId: previewPost.value.id,
        imageUrl: imageUrl,
        posted: previewPost.value.statistics.posted || '',
        rating: previewPost.value.statistics.rating || '',
        tags: previewPost.value.tagList || []
      });
      message.success('已添加到下载队列');
    } catch (error) {
      console.error('Failed to add download task:', error);
      message.error('添加下载任务失败');
    }
  }
}

function addTag(tag: string) {
  if (tag && !selectedTags.value.includes(tag)) {
    selectedTags.value.push(tag)
  }
}

function removeTag(tag: string) {
  selectedTags.value = selectedTags.value.filter(t => t !== tag)
}

// 键盘导航
function handleKeydown(e: KeyboardEvent) {
  if (!showPreview.value) return
  
  if (e.key === 'ArrowLeft') {
    prevImage()
  } else if (e.key === 'ArrowRight') {
    nextImage()
  } else if (e.key === 'Escape') {
    closePreview()
  }
}

onMounted(() => {
  searchPosts()
  window.addEventListener('keydown', handleKeydown)
})

watch([selectedTags, selectedRating], () => {
  if (!loading.value) {
    searchPosts(true)
  }
}, { deep: true })
</script>

<template>
  <div class="home-view">
    <!-- Search Bar -->
    <n-space vertical size="large">
      <n-space>
        <n-input
          v-model:value="searchInput"
          placeholder="输入标签搜索..."
          style="width: 400px"
          @keyup.enter="addTag(searchInput); searchInput = ''"
        />
        <n-select
          v-model:value="selectedRating"
          :options="ratingOptions"
          style="width: 150px"
        />
        <n-button type="primary" @click="searchPosts(true, true)">搜索</n-button>
      </n-space>
      
      <!-- Selected Tags -->
      <n-space v-if="selectedTags.length > 0">
        <n-tag
          v-for="tag in selectedTags"
          :key="tag"
          closable
          @close="removeTag(tag)"
        >
          {{ tag }}
        </n-tag>
      </n-space>
    </n-space>
    
    <!-- Results -->
    <n-spin :show="loading" style="margin-top: 20px;">
      <template v-if="galleryStore.posts.length > 0">
        <div class="post-grid">
          <div 
            v-for="(post, index) in galleryStore.posts" 
            :key="post.id" 
            class="post-card"
            @click="openPreview(index)"
          >
            <div class="post-thumbnail">
              <img
                :src="post.thumbnail || post.statistics.image"
                style="width: 100%; height: 100%; object-fit: cover;"
              />
            </div>
            <div class="post-info">
              <span style="font-size: 12px; color: #999;">#{{ post.id }}</span>
              <n-button size="small" type="primary" block @click.stop="downloadPost(post)">
                下载
              </n-button>
            </div>
          </div>
        </div>
        
        <n-space justify="center" style="margin-top: 20px;">
          <n-pagination
            v-model:page="galleryStore.currentPage"
            :page-count="galleryStore.totalPages"
            @update:page="() => searchPosts(false, true)"
          />
        </n-space>
      </template>
      <n-empty v-else description="搜索图片..." />
    </n-spin>
    
    <!-- Image Preview Modal -->
    <div v-if="showPreview" class="preview-overlay" @click.self="closePreview">
      <div class="preview-modal">
        <div class="preview-header">
          <span style="color: #fff;">#{{ currentPost?.id }} - {{ currentPreviewIndex + 1 }} / {{ galleryStore.posts.length }}</span>
          <n-button quaternary circle @click="closePreview">
            <template #icon>
              <n-icon :component="Close" color="#fff" />
            </template>
          </n-button>
        </div>
        <div class="preview-body">
          <!-- 左侧图片区域 -->
          <div class="preview-image-area">
            <div class="preview-container">
              <n-spin :show="previewLoading" style="min-height: 400px;">
                <img 
                  v-if="previewImageUrl" 
                  :src="previewImageUrl" 
                  style="max-width: 100%; max-height: 70vh; display: block; margin: 0 auto;"
                />
              </n-spin>
              
              <!-- Navigation -->
              <button 
                v-if="currentPreviewIndex > 0"
                class="nav-btn prev" 
                @click="prevImage"
              >
                <n-icon :component="ChevronBack" size="32" color="#fff" />
              </button>
              <button 
                v-if="currentPreviewIndex < galleryStore.posts.length - 1"
                class="nav-btn next" 
                @click="nextImage"
              >
                <n-icon :component="ChevronForward" size="32" color="#fff" />
              </button>
            </div>
          </div>
          
          <!-- 右侧信息区域 -->
          <div class="preview-sidebar">
            <div class="sidebar-info">
              <n-space v-if="previewPost">
                <n-tag :type="previewPost.statistics.rating === 'Safe' ? 'success' : previewPost.statistics.rating === 'Explicit' ? 'error' : 'warning'">
                  {{ previewPost.statistics.rating }}
                </n-tag>
                <n-tag>{{ previewPost.statistics.score }} 分</n-tag>
                <n-tag v-if="previewPost.statistics.size">{{ previewPost.statistics.size }}</n-tag>
              </n-space>
            </div>
            
            <div class="sidebar-tags" v-if="previewPost && previewPost.tagList.length > 0">
              <div class="tag-group" v-for="group in groupedTags" :key="group.type">
                <span class="tag-group-label">{{ group.label }}</span>
                <div class="tag-list">
                  <n-tag
                    v-for="tag in group.tags"
                    :key="tag.text"
                    :color="{ color: group.color, textColor: '#fff' }"
                    class="tag-item"
                    @click="handleTagClick(tag.text)"
                  >
                    {{ tag.text }}
                    <span class="tag-count" v-if="tag.count > 0">{{ formatCount(tag.count) }}</span>
                  </n-tag>
                </div>
              </div>
            </div>
            
            <div class="sidebar-actions">
              <n-button type="primary" block @click="downloadCurrentPost">
                下载原图
              </n-button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.home-view {
  padding: 0;
}

.post-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
}

@media (min-width: 900px) {
  .post-grid {
    grid-template-columns: repeat(6, 1fr);
  }
}

@media (min-width: 1400px) {
  .post-grid {
    grid-template-columns: repeat(7, 1fr);
  }
}

.post-card {
  border-radius: 8px;
  overflow: hidden;
  background: rgba(255, 255, 255, 0.06);
  transition: transform 0.2s ease, box-shadow 0.2s ease;
  display: flex;
  flex-direction: column;
  cursor: pointer;
}

.post-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.post-thumbnail {
  width: 100%;
  aspect-ratio: 1 / 1;
  overflow: hidden;
  background: transparent;
}

.post-thumbnail img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.post-info {
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.preview-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.85);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.preview-modal {
  background: rgba(30, 30, 30, 0.95);
  border-radius: 12px;
  max-width: 1400px;
  width: 95vw;
  max-height: 90vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.preview-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.preview-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.preview-image-area {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.3);
  position: relative;
}

.preview-container {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.nav-btn {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  background: rgba(0, 0, 0, 0.5);
  border: none;
  border-radius: 50%;
  width: 48px;
  height: 48px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.2s ease;
  z-index: 10;
}

.nav-btn:hover {
  background: rgba(0, 0, 0, 0.8);
}

.nav-btn.prev {
  left: 16px;
}

.nav-btn.next {
  right: 16px;
}

.preview-sidebar {
  width: 280px;
  display: flex;
  flex-direction: column;
  border-left: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(0, 0, 0, 0.2);
}

.sidebar-info {
  padding: 12px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.sidebar-tags {
  flex: 1;
  overflow-y: auto;
  padding: 12px 16px;
}

.sidebar-actions {
  padding: 12px 16px;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
}

.tag-group {
  margin-bottom: 16px;
}

.tag-group:last-child {
  margin-bottom: 0;
}

.tag-group-label {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
  display: block;
  margin-bottom: 8px;
  font-weight: 500;
}

.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.tag-item {
  cursor: pointer;
  transition: transform 0.15s ease, opacity 0.15s ease;
}

.tag-item:hover {
  transform: scale(1.05);
  opacity: 0.85;
}

.tag-count {
  margin-left: 4px;
  font-size: 10px;
  opacity: 0.7;
}
</style>
