<script setup lang="ts">
import { ref, computed } from 'vue';
import { NSkeleton, NIcon } from 'naive-ui';
import { FolderOutline } from '@vicons/ionicons5';
import { convertFileSrc } from '@tauri-apps/api/core';
import { imageBase64Cache } from '@/utils/lruCache';
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

export type DisplayItem =
  | (ImageInfo & { _type: 'image' })
  | (SubDirInfo & { _type: 'folder' });

const props = defineProps<{
  images: ImageInfo[];
  subdirs: SubDirInfo[];
  loadingImages: boolean;
  selectedKey: string | null;
}>();

const emit = defineEmits<{
  'open-preview': [index: number];
  'enter-subdir': [subdir: SubDirInfo];
}>();

const skeletonCount = ref(12);

const displayItems = computed<DisplayItem[]>(() => [
  ...props.subdirs.map((s) => ({ ...s, _type: 'folder' as const })),
  ...props.images.map((i) => ({ ...i, _type: 'image' as const })),
]);

function getImageSrc(path: string): string {
  if (!path) return '';
  return convertFileSrc(path.replace(/\\/g, '/'));
}

function handleImageError(event: Event, path: string) {
  const img = event.target as HTMLImageElement;
  if (!img || !path || imageBase64Cache.has(path)) return;
  import('@tauri-apps/api/core').then(({ invoke }) => {
    invoke<string>('get_local_image_base64', { path })
      .then((base64) => {
        imageBase64Cache.set(path, base64);
        img.src = base64;
      })
      .catch(() => {});
  });
}

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
        v-if="item._type === 'folder'"
        class="gallery-card folder-card"
        :data-image-path="item.thumbnail ?? ''"
        @click="emit('enter-subdir', item)"
      >
        <div class="folder-preview">
          <img
            v-if="item.thumbnail"
            :src="getImageSrc(item.thumbnail)"
            alt=""
            loading="lazy"
            @error="handleImageError($event, item.thumbnail)"
          />
          <n-icon v-else :size="48" color="#999">
            <FolderOutline />
          </n-icon>
        </div>
        <div class="card-filename">{{ item.name }} ({{ item.imageCount }})</div>
      </div>

      <div
        v-else
        class="gallery-card"
        :data-image-path="item.path"
        @click="emit('open-preview', index - props.subdirs.length)"
      >
        <img
          :src="getImageSrc(item.path)"
          :alt="item.name"
          loading="lazy"
          @error="handleImageError($event, item.path)"
        />
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
  aspect-ratio: 1;
  border-radius: 4px;
  overflow: hidden;
  background: #fff;
  cursor: pointer;
  transition: box-shadow 0.2s ease, z-index 0s;
  z-index: 0;
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

/* D-06: folder cards same as image cards — no text by default */
.gallery-card.folder-card .card-filename {
  opacity: 0;
}

.gallery-card.folder-card:hover .card-filename {
  opacity: 1;
}

/* D-06: No text labels visible by default */
.gallery-card img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

/* D-11: Folder preview area */
.folder-preview {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  background: rgba(0, 0, 0, 0.02);
}

.folder-preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  opacity: 0.85;
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
