<script setup lang="ts">
import { ref, onMounted } from 'vue'
import {
  NEmpty,
  NButton,
  NSpace,
  NText
} from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'
import { convertFileSrc } from '@tauri-apps/api/core'

interface ImageInfo {
  path: string
  name: string
}

const images = ref<ImageInfo[]>([])
const loading = ref(false)
const showPreview = ref(false)
const previewUrl = ref('')

function openPreview(url: string) {
  console.log('Opening preview:', url)
  previewUrl.value = url
  showPreview.value = true
}

async function loadImages() {
  loading.value = true
  try {
    const result = await invoke<string[]>('get_local_images')
    images.value = result.map(path => ({
      path,
      name: path.split(/[/\\]/).pop() || path
    }))
  } catch (error) {
    console.error('Failed to load images:', error)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadImages()
})
</script>

<template>
  <div class="gallery-view" @click="() => console.log('gallery-view clicked')">
    <n-space justify="space-between" style="margin-bottom: 16px;">
      <span style="font-size: 18px; font-weight: 500;">本地图库</span>
      <n-button @click="loadImages">刷新</n-button>
    </n-space>
    
    <div v-if="images.length > 0" class="image-grid">
      <div 
        v-for="(img, index) in images" 
        :key="index" 
        class="image-card"
        @click.stop="openPreview(convertFileSrc(img.path))"
      >
        <img
          :src="convertFileSrc(img.path)"
          style="width: 100%; height: 200px; object-fit: cover;"
        />
        <n-text 
          depth="3" 
          style="font-size: 12px; display: block; margin-top: 8px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;"
        >
          {{ img.name }}
        </n-text>
      </div>
    </div>
    <n-empty v-else description="暂无本地图片" />
  </div>
</template>

<style scoped>
.gallery-view {
  padding: 0;
}

.image-grid {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 12px;
}

@media (max-width: 1200px) {
  .image-grid {
    grid-template-columns: repeat(4, 1fr);
  }
}

@media (max-width: 900px) {
  .image-grid {
    grid-template-columns: repeat(3, 1fr);
  }
}

@media (max-width: 600px) {
  .image-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

.image-card {
  padding: 4px;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.05);
  cursor: pointer;
}
</style>
