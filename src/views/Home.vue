<script setup lang="ts">
import { ref, onMounted, watch, computed, nextTick } from 'vue'
import { onBeforeRouteLeave, useRoute } from 'vue-router'
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
  NCascader,
  useMessage
} from 'naive-ui'
import { ChevronBack, ChevronForward, Close, HeartOutline } from '@vicons/ionicons5'
import { useGalleryStore } from '@/stores/gallery'
import { useDownloadStore } from '@/stores/download'
import { useFavoriteTagsStore } from '@/stores/favoriteTags'
import { invoke } from '@tauri-apps/api/core'
import type { GelbooruPost, GelbooruTag } from '@/types'

const galleryStore = useGalleryStore()
const downloadStore = useDownloadStore()
const favoriteTagsStore = useFavoriteTagsStore()
const message = useMessage()
const route = useRoute()

const searchInput = ref('')
const isRestoring = ref(false)
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

// 作品标签
const copyrightTags = computed(() => {
  if (!previewPost.value?.tagList) return []
  return previewPost.value.tagList.filter(tag => tag.tagType.toLowerCase() === 'copyright')
})

// 角色标签
const characterTags = computed(() => {
  if (!previewPost.value?.tagList) return []
  return previewPost.value.tagList.filter(tag => tag.tagType.toLowerCase() === 'character')
})

// 其他标签分组（排除作品和角色）
const otherGroupedTags = computed(() => {
  if (!previewPost.value?.tagList) return []
  
  const groups: Record<string, { type: string; label: string; color: string; tags: GelbooruTag[] }> = {}
  
  for (const tag of previewPost.value.tagList) {
    const type = tag.tagType.toLowerCase()
    // 跳过作品和角色标签
    if (type === 'copyright' || type === 'character') continue
    
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
  
  // 按优先级排序
  const order = ['artist', 'general', 'metadata']
  return Object.values(groups).sort((a, b) => {
    return order.indexOf(a.type) - order.indexOf(b.type)
  })
})

// 收藏作品和角色标签
async function favoriteCopyrightAndCharacters() {
  if (!previewPost.value?.tagList) return
  
  try {
    // 先收藏作品标签（作为父标签）
    for (const tag of copyrightTags.value) {
      const tagName = tag.text.replace(/\s+/g, '_')
      await favoriteTagsStore.addParentTag(tagName, 'copyright')
    }
    
    // 再收藏角色标签（作为子标签）
    for (const copyrightTag of copyrightTags.value) {
      const parentName = copyrightTag.text.replace(/\s+/g, '_')
      // 查找父标签
      const group = favoriteTagsStore.findTagGroup(parentName)
      if (group) {
        for (const characterTag of characterTags.value) {
          const characterName = characterTag.text.replace(/\s+/g, '_')
          await favoriteTagsStore.addChildTag(characterName, 'character', group.parent.id)
        }
      }
    }
    
    message.success('已收藏作品和角色标签')
  } catch (error) {
    console.error('Failed to favorite tags:', error)
    message.error('收藏失败')
  }
}

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

// 级联选择器选项（作品 -> 角色）
const cascaderOptions = computed(() => {
  return favoriteTagsStore.tags
    .filter(group => group.parent.tagType === 'copyright')
    .map(group => ({
      label: group.parent.tag,
      value: group.parent.tag,
      children: group.children
        .filter(child => child.tagType === 'character')
        .map(child => ({
          label: child.tag,
          value: child.tag
        }))
    }))
})

const selectedCascaderValue = ref<string | string[] | null>(null)

// 处理级联选择
function handleCascaderChange(value: string | string[] | null) {
  if (!value) return
  
  const selectedValue = Array.isArray(value) ? value[value.length - 1] : value
  if (!selectedValue) return
  
  // 查找是否是子节点（角色）
  let isParentNode = false
  let parentValue = ''
  
  for (const option of cascaderOptions.value) {
    if (option.value === selectedValue) {
      // 选中的是父节点（作品）
      isParentNode = true
      break
    }
    if (option.children) {
      const child = option.children.find(c => c.value === selectedValue)
      if (child) {
        // 选中的是子节点（角色），记录父节点
        parentValue = option.value
        break
      }
    }
  }
  
  selectedCascaderValue.value = null
  
  if (isParentNode) {
    // 选中的是作品标签，替换搜索条件
    selectedTags.value = [selectedValue]
    searchPosts(true)
  } else {
    // 选中的是角色标签，追加作品和角色
    let added = false
    if (parentValue && !selectedTags.value.includes(parentValue)) {
      selectedTags.value.push(parentValue)
      added = true
    }
    if (!selectedTags.value.includes(selectedValue)) {
      selectedTags.value.push(selectedValue)
      added = true
    }
    if (added) {
      searchPosts(true)
    }
  }
}

async function searchPosts(resetPage = false) {
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
  
  loading.value = true
  try {
    const result = await invoke<{ postList: GelbooruPost[], tagList: GelbooruTag[], totalPages: number }>('search_posts', {
      tags: tags,
      page: galleryStore.currentPage
    })
    
    galleryStore.setPosts(result.postList)
    galleryStore.setTags(result.tagList)
    galleryStore.setTotalPages(result.totalPages)
    galleryStore.setSearchTags(tags)
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
  // 加载收藏标签
  favoriteTagsStore.loadTags()
  
  // 处理 query 参数（从收藏标签页跳转）
  if (route.query.tag) {
    const tag = route.query.tag as string
    if (!selectedTags.value.includes(tag)) {
      selectedTags.value.push(tag)
    }
    searchPosts(true)
    return
  }
  
  // 尝试恢复页面状态
  const savedState = galleryStore.restorePageState()
  if (savedState) {
    isRestoring.value = true
    selectedTags.value = savedState.selectedTags
    selectedRating.value = savedState.selectedRating
    galleryStore.setPosts(savedState.posts)
    galleryStore.setTags(savedState.tags)
    galleryStore.setTotalPages(savedState.totalPages)
    galleryStore.setSearchTags(savedState.searchTags)
    // 必须在 setSearchTags 之后设置，否则会被重置为1
    galleryStore.currentPage = savedState.currentPage
    // 使用 setTimeout 确保 watch 回调先执行
    setTimeout(() => {
      isRestoring.value = false
    }, 0)
    console.log('[Home] Restored page state')
  } else {
    // 没有保存的状态，执行初始搜索
    searchPosts()
  }
  window.addEventListener('keydown', handleKeydown)
})

// 离开页面前保存状态
onBeforeRouteLeave(() => {
  galleryStore.savePageState(selectedTags.value, selectedRating.value)
})

watch([selectedTags, selectedRating], () => {
  if (isRestoring.value || loading.value) {
    return
  }
  searchPosts(true)
}, { deep: true, flush: 'sync' })
</script>

<template>
  <div class="home-view">
    <!-- Quick Tag Selector -->
    <div v-if="cascaderOptions.length > 0" class="quick-tag-selector">
      <span class="selector-label">快速选择：</span>
      <n-cascader
        v-model:value="selectedCascaderValue"
        :options="cascaderOptions"
        placeholder="选择作品/角色"
        check-strategy="all"
        filterable
        clearable
        style="width: 300px"
        @update:value="handleCascaderChange"
      />
    </div>
    
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
        <n-button type="primary" @click="searchPosts(true)">搜索</n-button>
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
            @update:page="() => searchPosts(false)"
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
            
            <!-- 作品和角色标签区域 -->
            <div class="sidebar-tags" v-if="previewPost && previewPost.tagList.length > 0">
              <!-- 作品和角色标签（独立显示） -->
              <div v-if="copyrightTags.length > 0 || characterTags.length > 0" class="main-tags-section">
                <div class="main-tags-header">
                  <span class="main-tags-label">作品/角色</span>
                  <n-button 
                    v-if="copyrightTags.length > 0 || characterTags.length > 0"
                    size="tiny" 
                    type="primary" 
                    quaternary
                    @click="favoriteCopyrightAndCharacters"
                  >
                    <template #icon>
                      <n-icon :component="HeartOutline" size="14" />
                    </template>
                    收藏
                  </n-button>
                </div>
                <div class="main-tags-list">
                  <!-- 作品标签 -->
                  <n-tag
                    v-for="tag in copyrightTags"
                    :key="tag.text"
                    :color="{ color: tagTypeConfig.copyright.color, textColor: '#fff' }"
                    class="main-tag copyright-tag"
                    @click="handleTagClick(tag.text)"
                  >
                    {{ tag.text }}
                  </n-tag>
                  <!-- 角色标签 -->
                  <n-tag
                    v-for="tag in characterTags"
                    :key="tag.text"
                    :color="{ color: tagTypeConfig.character.color, textColor: '#fff' }"
                    class="main-tag character-tag"
                    @click="handleTagClick(tag.text)"
                  >
                    {{ tag.text }}
                  </n-tag>
                </div>
              </div>
              
              <!-- 其他标签 -->
              <div class="other-tags-section" v-if="otherGroupedTags.length > 0">
                <div class="tag-group" v-for="group in otherGroupedTags" :key="group.type">
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

.quick-tag-selector {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  margin-bottom: 16px;
  background: rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.selector-label {
  font-size: 14px;
  font-weight: 500;
  color: #999;
  white-space: nowrap;
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

.main-tags-section {
  margin-bottom: 16px;
  padding: 12px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 8px;
}

.main-tags-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.main-tags-label {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.6);
  font-weight: 500;
}

.main-tags-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.main-tag {
  cursor: pointer;
  transition: transform 0.15s ease, opacity 0.15s ease;
  font-size: 13px;
  padding: 4px 10px;
}

.main-tag:hover {
  transform: scale(1.05);
  opacity: 0.85;
}

.other-tags-section {
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  padding-top: 12px;
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
  max-width: 150px;
}

.tag-item :deep(.n-tag__content) {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
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
