<script setup lang="ts">
import { computed } from 'vue';
import { NIcon } from 'naive-ui';
import { FolderOutline } from '@vicons/ionicons5';
import { convertFileSrc } from '@tauri-apps/api/core';

export interface ImageInfo {
  path: string;
  name: string;
}

export interface SubDirInfo {
  path: string;
  name: string;
  imageCount: number;
  thumbnail?: string;
}

const props = defineProps<{
  images: ImageInfo[];
  loadingImages: boolean;
  selectedKey: string | null;
}>();

const emit = defineEmits<{
  'open-preview': [index: number];
  'enter-subdir': [subdir: SubDirInfo];
}>();

// 直接将 path 转换为 asset URL，供模板使用
function cardSrc(path: string): string {
  return convertFileSrc(path.replace(/\\/g, '/'));
}

// 暴露图片数量
defineExpose({ imageCount: computed(() => props.images.length) });

// Simple computed: MasonryWall receives the same reactive array as props.images.
// On load-more: images.value.push() → props.images array grows → computed re-runs → MasonryWall renders new items.
// On reset/switch: images.value = newArray → array ref changes → computed re-runs → full rebuild.
const displayItems = computed(() =>
  props.images.map((i) => ({ ...i, _type: 'image' as const }))
);
</script>

<template>
  <!-- Pure CSS masonry: column-count does NOT recreate DOM on item changes → scroll position stable -->
  <div v-if="displayItems.length > 0" class="gallery-cards content-grid" data-gallery-cards>
    <div
      v-for="(item, index) in displayItems"
      :key="item.path"
      class="gallery-card"
      :data-image-path="item.path"
      @click="emit('open-preview', index)"
    >
      <!-- 占位背景 + 真实图片叠加 -->
      <div class="card-image-wrapper">
        <div class="card-placeholder" />
        <img
          :src="cardSrc(item.path)"
          :alt="item.name"
          class="card-img card-img-loaded"
          @error="($event.target as HTMLImageElement | null)?.remove()"
        />
      </div>
      <div class="card-filename">{{ item.name }}</div>
    </div>
  </div>

  <!-- D-10: Empty state -->
  <div v-if="selectedKey && displayItems.length === 0 && !loadingImages" class="empty-state">
    <n-icon :size="64" depth="4">
      <FolderOutline />
    </n-icon>
    <p class="empty-text">该目录下暂无图片</p>
  </div>
</template>

<style scoped>
/* D-04: White #fff, border-radius 4px, no border */
.gallery-card {
  position: relative;
  border-radius: 4px;
  overflow: hidden;
  background: #fff;
  cursor: pointer;
  transition: box-shadow 0.2s ease, z-index 0s;
  z-index: 0;
  min-height: 100px;
}

/* Pure CSS masonry: column-count keeps all DOM nodes stable on item changes */
.gallery-cards {
  columns: 160px 3;
  column-gap: 4px;
}

/* Break-inside ensures cards don't split across columns */
.gallery-card {
  break-inside: avoid;
  margin-bottom: 4px;
}

/* 图片容器：维持占位高度直到图片加载完成 */
.card-image-wrapper {
  position: relative;
  width: 100%;
  min-height: 120px;
  background: rgba(0, 0, 0, 0.04);
}

/* 占位图：静态纯色，避免 animated gradient 持续触发 paint */
.card-placeholder {
  position: absolute;
  inset: 0;
  background: #e5e5e5;
}

/* 真实图片：opacity 淡入过渡，轻量 GPU 合成 */
.card-img {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  opacity: 0;
  transition: opacity 0.3s ease;
  /* compositor-only 属性，滚动时不会触发 layout/paint */
  will-change: opacity;
}

.card-img-loaded {
  opacity: 1;
}

/* D-04: hover box-shadow */
.gallery-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  z-index: 10;
}

/* D-07: bottom gradient overlay via ::after */
.gallery-card::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 50%;
  background: linear-gradient(transparent, rgba(0, 0, 0, 0.6));
  opacity: 0;
  transition: opacity 0.2s;
  pointer-events: none;
}

.gallery-card:hover::after {
  opacity: 1;
}

/* D-07: filename on hover only */
.card-filename {
  position: absolute;
  bottom: 8px;
  left: 8px;
  right: 8px;
  color: #fff;
  font-size: 12px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  opacity: 0;
  transition: opacity 0.2s;
  z-index: 1;
}

.gallery-card:hover .card-filename {
  opacity: 1;
}

/* D-10: Empty state centering */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 60vh;
  gap: 16px;
}

.empty-text {
  color: #999;
  font-size: 14px;
  margin: 0;
}
</style>