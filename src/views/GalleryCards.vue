<script setup lang="ts">
import { ref, computed } from 'vue';
import { NSkeleton, NIcon } from 'naive-ui';
import { FolderOutline } from '@vicons/ionicons5';
import { MasonryWall } from '@yeger/vue-masonry-wall';

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

const skeletonCount = ref(12);

// 图片 src 状态：src 为空时显示占位图，进入视口后加载真实 URL
const imageSrcMap = ref<Map<string, string>>(new Map());

// 初始化时所有图片 src 为空，由 Gallery.vue 的 IntersectionObserver 驱动加载
function getCardSrc(path: string): string {
  return imageSrcMap.value.get(path) ?? '';
}

function setCardSrc(path: string, src: string) {
  imageSrcMap.value.set(path, src);
}

// 暴露图片路径列表，供 Gallery.vue 挂载 IntersectionObserver
defineExpose({ getCardSrc, setCardSrc, imageCount: computed(() => props.images.length) });

const displayItems = computed(() =>
  props.images.map((i) => ({ ...i, _type: 'image' as const }))
);
</script>

<template>
  <!-- D-09: NSkeleton loading state -->
  <div v-if="loadingImages" class="content-grid">
    <div v-for="i in skeletonCount" :key="i" class="skeleton-card">
      <n-skeleton :height="160" width="100%" :sharp="false" />
    </div>
  </div>

  <!-- Masonry grid: replaces CSS Grid auto-fill with @yeger/vue-masonry-wall -->
  <MasonryWall
    v-else-if="displayItems.length > 0"
    :items="displayItems"
    :column-width="160"
    :gap="4"
    :min-columns="1"
    class="content-grid"
  >
    <template #default="{ item, index }">
      <div
        class="gallery-card"
        :data-image-path="item.path"
        @click="emit('open-preview', index)"
      >
        <!-- 占位背景 + 真实图片叠加 -->
        <div class="card-image-wrapper">
          <div class="card-placeholder" />
          <img
            v-if="getCardSrc(item.path)"
            :src="getCardSrc(item.path)"
            :alt="item.name"
            class="card-img card-img-loaded"
            @error="getCardSrc(item.path) && (imageSrcMap.delete(item.path))"
          />
        </div>
        <div class="card-filename">{{ item.name }}</div>
      </div>
    </template>
  </MasonryWall>

  <!-- D-10: Empty state -->
  <div v-else-if="selectedKey && !loadingImages" class="empty-state">
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

/* MasonryWall owns this layout */
.masonry-wall {
  width: 100%;
}

/* 图片容器：维持占位高度直到图片加载完成 */
.card-image-wrapper {
  position: relative;
  width: 100%;
  min-height: 120px;
  background: rgba(0, 0, 0, 0.04);
}

/* 占位图：灰色矩形，直到真实图片加载完成才隐藏 */
.card-placeholder {
  position: absolute;
  inset: 0;
  background: linear-gradient(135deg, #e8e8e8 25%, #f5f5f5 50%, #e8e8e8 75%);
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
}

@keyframes shimmer {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

/* 真实图片：从模糊过渡到清晰 */
.card-img {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  filter: blur(8px);
  transform: scale(1.05); /* 轻微放大掩盖模糊边界 */
  transition: filter 0.4s ease, transform 0.4s ease;
}

.card-img-loaded {
  filter: none;
  transform: none;
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

/* D-09: Skeleton card matching card dimensions */
.skeleton-card {
  border-radius: 4px;
  overflow: hidden;
  background: #fff;
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
