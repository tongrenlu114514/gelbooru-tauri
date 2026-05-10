<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, nextTick, watch } from 'vue';
import {
  NButton,
  NSpace,
  NText,
  NIcon,
  NModal,
  NLayout,
  NLayoutContent,
  NBreadcrumb,
  NBreadcrumbItem,
  useMessage,
  useDialog,
  NSpin,
} from 'naive-ui';
import {
  RefreshOutline,
  FolderOpenOutline,
  TrashOutline,
  ChevronBackOutline,
  ChevronForwardOutline,
  ChevronUpOutline,
} from '@vicons/ionicons5';
import { invoke } from '@tauri-apps/api/core';
import { convertFileSrc } from '@tauri-apps/api/core';
import { imageBase64Cache } from '@/utils/lruCache';
import GalleryCards from './GalleryCards.vue';
import type { ImageInfo, SubDirInfo } from './GalleryCards.vue';
import { useSettingsStore } from '@/stores/settings';

const message = useMessage();
const dialog = useDialog();
const settingsStore = useSettingsStore();

// D-05: convertFileSrc primary (base64 fallback only on error)
function getImageSrc(path: string): string {
  if (!path) return '';
  return convertFileSrc(path.replace(/\\/g, '/'));
}

// IntersectionObserver — Phase 3 memory leak fix, preserved intact
const observerRef = ref<IntersectionObserver | null>(null);

function observeCallback(entries: IntersectionObserverEntry[]) {
  entries.forEach((entry) => {
    if (entry.isIntersecting) {
      const path = (entry.target as HTMLElement).dataset.imagePath;
      if (path) loadImageBase64(path);
    }
  });
}

async function loadImageBase64(path: string) {
  if (imageBase64Cache.has(path)) return;
  try {
    const base64 = await invoke<string>('get_local_image_base64', { path });
    imageBase64Cache.set(path, base64);
  } catch {
    // Silent fallback — image already shows convertFileSrc URL
  }
}

function loadVisibleImages() {
  const grid = document.querySelector('.content-grid');
  if (!grid || !observerRef.value) return;
  const cards = grid.querySelectorAll<HTMLElement>('[data-image-path]');
  cards.forEach((card) => observerRef.value!.observe(card));
}

const selectedKey = ref<string | null>(null);
const subdirs = ref<SubDirInfo[]>([]);
const images = ref<ImageInfo[]>([]);
const loadingTree = ref(false);
const loadingImages = ref(false);

const showPreview = ref(false);
const previewIndex = ref(0);

const currentImage = computed(() => images.value[previewIndex.value]);

// Breadcrumb segments: path relative to downloadPath, split into segments
const breadcrumbSegments = computed(() => {
  if (!selectedKey.value || !settingsStore.downloadPath) return [];
  const normalizedPath = selectedKey.value.replace(/\\/g, '/');
  const normalizedRoot = settingsStore.downloadPath.replace(/\\/g, '/');
  if (!normalizedPath.startsWith(normalizedRoot)) return [];
  const relative = normalizedPath.slice(normalizedRoot.length).replace(/^\/+|\/+$/g, '');
  if (!relative) return [];
  return relative.split('/');
});

// Parent path: strip last path segment
const parentPath = computed(() => {
  if (!selectedKey.value) return null;
  const parts = selectedKey.value.replace(/\\/g, '/').split('/');
  parts.pop();
  return parts.join('/') || null;
});

function openPreview(index: number) {
  previewIndex.value = index;
  showPreview.value = true;
}

function prevImage() {
  if (previewIndex.value > 0) previewIndex.value--;
}

function nextImage() {
  if (previewIndex.value < images.value.length - 1) previewIndex.value++;
}

function handleKeydown(e: KeyboardEvent) {
  if (!showPreview.value) return;
  if (e.key === 'ArrowLeft') prevImage();
  if (e.key === 'ArrowRight') nextImage();
  if (e.key === 'Escape') showPreview.value = false;
}

async function loadImagesForDirectory(dirPath: string) {
  loadingImages.value = true;
  try {
    const result = await invoke<{ subdirs: SubDirInfo[]; images: ImageInfo[]; total: number }>(
      'get_directory_images',
      { dirPath }
    );
    subdirs.value = result.subdirs;
    images.value = result.images;
    await nextTick();
    loadVisibleImages();
  } catch (error) {
    console.error('Failed to load images:', error);
    subdirs.value = [];
    images.value = [];
  } finally {
    loadingImages.value = false;
  }
}

function handleImageError(event: Event, path: string) {
  const img = event.target as HTMLImageElement;
  if (!img || !path || imageBase64Cache.has(path)) return;
  invoke<string>('get_local_image_base64', { path })
    .then((base64) => {
      imageBase64Cache.set(path, base64);
      img.src = base64;
    })
    .catch(() => {});
}

async function enterSubdir(subdir: SubDirInfo) {
  if (selectedKey.value === subdir.path) return; // Same folder — no-op
  selectedKey.value = subdir.path;
  await loadImagesForDirectory(subdir.path);
  await nextTick();
  scrollToFirstCard();
}

// Smooth-scroll to first image card in the masonry grid, but only if not already visible
function scrollToFirstCard() {
  const grid = document.querySelector('.content-grid');
  if (!grid) return;
  const firstCard = grid.querySelector<HTMLElement>('[data-image-path]');
  if (!firstCard) return;
  const rect = firstCard.getBoundingClientRect();
  const inViewport = rect.top >= 0 && rect.bottom <= window.innerHeight;
  if (!inViewport) {
    firstCard.scrollIntoView({ behavior: 'smooth', block: 'start' });
  }
}

// Navigate to ancestor folder via breadcrumb click, then scroll to first image in viewport
function handleBreadcrumbClick(index: number) {
  const prefix = breadcrumbSegments.value.slice(0, index + 1).join('/');
  const normalizedRoot = settingsStore.downloadPath.replace(/\\/g, '/');
  const targetPath = `${normalizedRoot}/${prefix}`;
  if (targetPath === selectedKey.value) return; // Already in this folder — no-op
  const targetSubdir: SubDirInfo = {
    path: targetPath,
    name: breadcrumbSegments.value[index],
    imageCount: 0,
  };
  enterSubdir(targetSubdir);
}

async function deleteImage(index: number) {
  const img = images.value[index];
  dialog.warning({
    title: '确认删除',
    content: `确定要删除 "${img.name}" 吗？此操作不可撤销。`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await invoke('delete_image', { path: img.path });
        images.value.splice(index, 1);
        if (showPreview.value && previewIndex.value >= images.value.length) {
          previewIndex.value = Math.max(0, images.value.length - 1);
        }
        if (images.value.length === 0) showPreview.value = false;
        message.success('删除成功');
        await refresh();
      } catch (error) {
        message.error(`删除失败: ${error}`);
      }
    },
  });
}

function goUp() {
  if (!parentPath.value) return;
  const upSubdir: SubDirInfo = { path: parentPath.value, name: '..', imageCount: 0 };
  enterSubdir(upSubdir);
}

async function refresh() {
  observerRef.value?.disconnect();
  selectedKey.value = null;
  subdirs.value = [];
  images.value = [];
  await loadTree();
}

async function loadTree() {
  loadingTree.value = true;
  try {
    // Load download directory on mount
    const rootDir = settingsStore.downloadPath;
    if (rootDir) {
      await loadImagesForDirectory(rootDir);
      selectedKey.value = rootDir;
    } else {
      selectedKey.value = null;
    }
  } catch (error) {
    console.error('Failed to load directory:', error);
  } finally {
    loadingTree.value = false;
  }
}

onMounted(() => {
  loadTree();
  window.addEventListener('keydown', handleKeydown);
  observerRef.value = new IntersectionObserver(observeCallback, {
    root: null,
    rootMargin: '200px',
    threshold: 0.01,
  });
});

onUnmounted(() => {
  observerRef.value?.disconnect();
});

watch(
  () => settingsStore.downloadPath,
  () => refresh()
);

defineExpose({ loadVisibleImages });
</script>

<template>
  <div class="gallery-view">
    <n-space justify="space-between" align="center" style="margin-bottom: 16px">
      <span style="font-size: 18px; font-weight: 500"> 本地图库 </span>
      <n-button :loading="loadingTree" @click="refresh">
        <template #icon>
          <n-icon><RefreshOutline /></n-icon>
        </template>
        刷新
      </n-button>
    </n-space>

    <n-layout style="height: calc(100vh - 140px)">
      <n-layout-content content-style="padding: 12px">
        <!-- Breadcrumb navigation: replaces flat .path-bar -->
        <div v-if="breadcrumbSegments.length > 0" class="breadcrumb-bar">
          <n-breadcrumb>
            <n-breadcrumb-item
              v-for="(segment, i) in breadcrumbSegments"
              :key="i"
              :clickable="i < breadcrumbSegments.length - 1"
              @click="i < breadcrumbSegments.length - 1 && handleBreadcrumbClick(i)"
            >
              <n-icon :size="14" color="#f0a020" style="margin-right: 4px">
                <FolderOpenOutline />
              </n-icon>
              {{ segment }}
            </n-breadcrumb-item>
          </n-breadcrumb>
        </div>

        <!-- Folder list: flat horizontal navigation above image grid -->
        <div v-if="selectedKey !== null" class="folder-list">
          <n-spin :show="loadingImages" size="small">
            <div class="folder-list-inner">
              <!-- ".." up navigation -->
              <div
                v-if="parentPath"
                class="folder-item folder-up"
                @click="goUp"
              >
                <n-icon :size="14" color="#999">
                  <ChevronUpOutline />
                </n-icon>
                <span class="folder-name">..</span>
              </div>

              <!-- Divider before subdirs -->
              <span v-if="parentPath && subdirs.length > 0" class="folder-divider">|</span>

              <!-- Subdir entries -->
              <template v-for="(subdir, i) in subdirs" :key="subdir.path">
                <div
                  class="folder-item"
                  :class="{ active: selectedKey === subdir.path }"
                  @click="enterSubdir(subdir)"
                >
                  <n-icon :size="14" color="#f0a020">
                    <FolderOpenOutline />
                  </n-icon>
                  <span class="folder-name">{{ subdir.name }}</span>
                  <span class="folder-count">{{ subdir.imageCount }}</span>
                </div>
                <span v-if="i < subdirs.length - 1" class="folder-divider">|</span>
              </template>
            </div>
          </n-spin>
        </div>

        <!-- Empty state before any folder is selected -->
        <div v-if="selectedKey === null && !loadingTree" class="no-folder-hint">
          <n-icon :size="40" depth="4"><FolderOpenOutline /></n-icon>
          <p>从上方列表选择一个文件夹开始浏览</p>
        </div>

        <!-- Image card grid -->
        <GalleryCards
          :images="images"
          :subdirs="subdirs"
          :loading-images="loadingImages"
          :selected-key="selectedKey"
          @open-preview="openPreview"
          @enter-subdir="enterSubdir"
        />
      </n-layout-content>
    </n-layout>

    <!-- D-08: Preview modal with ArrowLeft/Right keyboard navigation -->
    <n-modal
      v-model:show="showPreview"
      preset="card"
      style="width: auto; max-width: 90vw; max-height: 90vh"
    >
      <template #header>
        <n-text>{{ currentImage?.name }}</n-text>
      </template>
      <div class="preview-container">
        <img
          v-if="currentImage"
          :src="getImageSrc(currentImage.path)"
          style="max-width: 80vw; max-height: 70vh; object-fit: contain"
          @error="handleImageError($event, currentImage.path)"
        />
        <div class="preview-nav">
          <n-button quaternary circle :disabled="previewIndex === 0" @click="prevImage">
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
            @click="
              showPreview = false;
              deleteImage(previewIndex);
            "
          >
            <template #icon>
              <n-icon><TrashOutline /></n-icon>
            </template>
            删除
          </n-button>
        </div>
      </div>
    </n-modal>
  </div>
</template>

<style scoped>
.gallery-view {
  padding: 0;
}

/* Replaces .path-bar — flat breadcrumb bar per UI-SPEC */
.breadcrumb-bar {
  display: flex;
  align-items: center;
  margin-bottom: 8px;
  padding: 0;
}

.folder-list {
  margin-bottom: 8px;
  min-height: 32px;
}

.folder-list-inner {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 2px;
}

.folder-item {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.15s;
}

.folder-item:hover {
  background: #f0f0f0;
}

.folder-item.active {
  background: #e8f0fe;
}

.folder-up {
  color: #666;
}

.folder-name {
  font-size: 13px;
  color: #333;
}

.folder-count {
  font-size: 11px;
  color: #999;
  background: #f5f5f5;
  padding: 1px 5px;
  border-radius: 8px;
}

.folder-divider {
  color: #ddd;
  font-size: 12px;
  user-select: none;
}

.no-folder-hint {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  height: 200px;
  color: #999;
  font-size: 14px;
}

.no-folder-hint p {
  margin: 0;
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