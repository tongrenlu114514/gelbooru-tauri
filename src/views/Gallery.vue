<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import {
  NEmpty,
  NButton,
  NSpace,
  NText,
  NSpin,
  NIcon,
  NList,
  NListItem,
  NModal,
  useMessage,
  useDialog
} from 'naive-ui'
import { RefreshOutline, OpenOutline, FolderOpenOutline, TrashOutline, ChevronBackOutline, ChevronForwardOutline } from '@vicons/ionicons5'
import { invoke } from '@tauri-apps/api/core'
import { convertFileSrc } from '@tauri-apps/api/core'

const message = useMessage()
const dialog = useDialog()

interface ImageInfo {
  path: string
  name: string
}

interface PaginatedImages {
  images: ImageInfo[]
  total: number
  has_more: boolean
}

const images = ref<ImageInfo[]>([])
const loading = ref(false)
const currentPage = ref(1)
const totalImages = ref(0)
const hasMore = ref(false)
const PAGE_SIZE = 100

const showPreview = ref(false)
const previewIndex = ref(0)

const currentImage = computed(() => images.value[previewIndex.value])

function openPreview(index: number) {
  previewIndex.value = index
  showPreview.value = true
}

function prevImage() {
  if (previewIndex.value > 0) {
    previewIndex.value--
  }
}

function nextImage() {
  if (previewIndex.value < images.value.length - 1) {
    previewIndex.value++
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (!showPreview.value) return
  if (e.key === 'ArrowLeft') prevImage()
  if (e.key === 'ArrowRight') nextImage()
  if (e.key === 'Escape') showPreview.value = false
}

async function openImage(path: string) {
  try {
    await invoke('open_file', { path })
  } catch (error) {
    console.error('Failed to open image:', error)
  }
}

async function openFolder(path: string) {
  try {
    const lastSep = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'))
    const folderPath = lastSep > 0 ? path.substring(0, lastSep) : path
    await invoke('open_file', { path: folderPath })
  } catch (error) {
    console.error('Failed to open folder:', error)
  }
}

async function deleteImage(index: number) {
  const img = images.value[index]
  dialog.warning({
    title: '确认删除',
    content: `确定要删除 "${img.name}" 吗？此操作不可撤销。`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await invoke('delete_image', { path: img.path })
        images.value.splice(index, 1)
        totalImages.value--
        if (showPreview.value && previewIndex.value >= images.value.length) {
          previewIndex.value = Math.max(0, images.value.length - 1)
        }
        if (images.value.length === 0) {
          showPreview.value = false
        }
        message.success('删除成功')
      } catch (error) {
        message.error(`删除失败: ${error}`)
      }
    }
  })
}

async function loadImages(page: number = 1, append: boolean = false) {
  if (loading.value) return
  loading.value = true
  
  try {
    const result = await invoke<PaginatedImages>('get_local_images', {
      page,
      pageSize: PAGE_SIZE
    })
    
    if (append) {
      images.value.push(...result.images)
    } else {
      images.value = result.images
    }
    
    currentPage.value = page
    totalImages.value = result.total
    hasMore.value = result.has_more
  } catch (error) {
    console.error('Failed to load images:', error)
  } finally {
    loading.value = false
  }
}

async function loadMore() {
  if (!hasMore.value || loading.value) return
  await loadImages(currentPage.value + 1, true)
}

async function refresh() {
  currentPage.value = 1
  images.value = []
  await loadImages(1)
}

onMounted(() => {
  loadImages()
  window.addEventListener('keydown', handleKeydown)
})
</script>

<template>
  <div class="gallery-view">
    <n-space justify="space-between" align="center" style="margin-bottom: 16px;">
      <span style="font-size: 18px; font-weight: 500;">
        本地图库
        <n-text depth="3" style="font-size: 14px; margin-left: 8px;">
          (共 {{ totalImages }} 张)
        </n-text>
      </span>
      <n-button @click="refresh" :loading="loading && currentPage === 1">
        <template #icon>
          <n-icon><RefreshOutline /></n-icon>
        </template>
        刷新
      </n-button>
    </n-space>
    
    <n-spin :show="loading && currentPage === 1">
      <n-list v-if="images.length > 0" hoverable clickable>
        <n-list-item v-for="(img, index) in images" :key="index" @click="openPreview(index)">
          <template #prefix>
            <n-text depth="3" style="min-width: 40px; text-align: right;">
              {{ (currentPage - 1) * PAGE_SIZE + index + 1 }}
            </n-text>
          </template>
          <n-text style="flex: 1; cursor: pointer;">{{ img.name }}</n-text>
          <template #suffix>
            <div class="action-buttons">
              <n-button size="small" quaternary @click.stop="openImage(img.path)">
                <template #icon>
                  <n-icon><OpenOutline /></n-icon>
                </template>
              </n-button>
              <n-button size="small" quaternary @click.stop="openFolder(img.path)">
                <template #icon>
                  <n-icon><FolderOpenOutline /></n-icon>
                </template>
              </n-button>
              <n-button size="small" quaternary @click.stop="deleteImage(index)">
                <template #icon>
                  <n-icon><TrashOutline /></n-icon>
                </template>
              </n-button>
            </div>
          </template>
        </n-list-item>
      </n-list>
      <n-empty v-else-if="!loading" description="暂无本地图片" />
    </n-spin>
    
    <n-modal v-model:show="showPreview" preset="card" style="width: auto; max-width: 90vw; max-height: 90vh;">
      <template #header>
        <n-text>{{ currentImage?.name }}</n-text>
      </template>
      <div class="preview-container">
        <img 
          v-if="currentImage" 
          :src="convertFileSrc(currentImage.path)" 
          style="max-width: 80vw; max-height: 70vh; object-fit: contain;"
        />
        <div class="preview-nav">
          <n-button 
            quaternary 
            circle 
            :disabled="previewIndex === 0"
            @click="prevImage"
          >
            <template #icon>
              <n-icon :size="24"><ChevronBackOutline /></n-icon>
            </template>
          </n-button>
          <n-text depth="3">{{ previewIndex + 1 }} / {{ images.length }}</n-text>
          <n-button 
            quaternary 
            circle 
            :disabled="previewIndex === images.length - 1"
            @click="nextImage"
          >
            <template #icon>
              <n-icon :size="24"><ChevronForwardOutline /></n-icon>
            </template>
          </n-button>
          <n-button 
            type="error" 
            quaternary
            @click="showPreview = false; deleteImage(previewIndex)"
          >
            <template #icon>
              <n-icon><TrashOutline /></n-icon>
            </template>
            删除
          </n-button>
        </div>
      </div>
    </n-modal>
    
    <div v-if="hasMore" style="text-align: center; margin-top: 24px;">
      <n-button 
        @click="loadMore" 
        :loading="loading && currentPage > 1"
        type="primary"
        ghost
      >
        加载更多 (已加载 {{ images.length }} / {{ totalImages }} 张)
      </n-button>
    </div>
  </div>
</template>

<style scoped>
.gallery-view {
  padding: 0;
}

.action-buttons {
  display: inline-flex;
  gap: 4px;
  flex-wrap: nowrap;
}

.preview-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.preview-nav {
  display: flex;
  align-items: center;
  gap: 16px;
}
</style>
