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
  useMessage,
  useDialog,
} from 'naive-ui';
import {
  RefreshOutline,
  FolderOpenOutline,
  TrashOutline,
  ChevronBackOutline,
  ChevronForwardOutline,
} from '@vicons/ionicons5';
import { invoke } from '@tauri-apps/api/core';
import { convertFileSrc } from '@tauri-apps/api/core';
import { imageBase64Cache } from '@/utils/lruCache';
import { useSettingsStore } from '@/stores/settings';
import GallerySidebar from './GallerySidebar.vue';
import GalleryCards from './GalleryCards.vue';
import type { ImageInfo, SubDirInfo } from './GalleryCards.vue';

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

watch(
  () => settingsStore.downloadPath,
  () => refresh()
);

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

interface TreeNode {
  key: string;
  label: string;
  path: string;
  isLeaf: boolean;
  imageCount: number;
  children?: TreeNode[];
  thumbnail?: string;
}

const treeData = ref<TreeNode[]>([]);
const selectedKey = ref<string | null>(null);
const subdirs = ref<SubDirInfo[]>([]);
const images = ref<ImageInfo[]>([]);
const loadingTree = ref(false);
const loadingImages = ref(false);

const showPreview = ref(false);
const previewIndex = ref(0);

const currentImage = computed(() => images.value[previewIndex.value]);
const currentPath = computed(() => selectedKey.value || '');

async function openCurrentFolder() {
  if (!currentPath.value) return;
  try {
    await invoke('open_file', { path: currentPath.value });
  } catch (error) {
    console.error('Failed to open folder:', error);
  }
}

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

function findNodeByKey(nodes: TreeNode[], key: string): TreeNode | null {
  for (const node of nodes) {
    if (node.key === key) return node;
    if (node.children) {
      const found = findNodeByKey(node.children, key);
      if (found) return found;
    }
  }
  return null;
}

function handleTreeSelect(key: string) {
  selectedKey.value = key;
  const node = findNodeByKey(treeData.value, key);
  if (node) {
    loadImagesForDirectory(node.path);
  } else {
    images.value = [];
    subdirs.value = [];
  }
}

async function loadTree() {
  loadingTree.value = true;
  try {
    treeData.value = await invoke<TreeNode[]>('get_directory_tree', {});
  } catch (error) {
    console.error('Failed to load directory tree:', error);
  } finally {
    loadingTree.value = false;
  }
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

function enterSubdir(subdir: SubDirInfo) {
  selectedKey.value = subdir.path;
  loadImagesForDirectory(subdir.path);
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
        await loadTree();
      } catch (error) {
        message.error(`删除失败: ${error}`);
      }
    },
  });
}

async function refresh() {
  observerRef.value?.disconnect();
  selectedKey.value = null;
  subdirs.value = [];
  images.value = [];
  await loadTree();
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

    <n-layout has-sider style="height: calc(100vh - 140px)">
      <GallerySidebar
        :tree-data="treeData"
        :selected-key="selectedKey"
        :loading-tree="loadingTree"
        @select="handleTreeSelect"
      />

      <n-layout-content content-style="padding: 12px">
        <div v-if="selectedKey" class="path-bar" @click="openCurrentFolder">
          <n-icon :size="16"><FolderOpenOutline /></n-icon>
          <span class="path-text">{{ currentPath }}</span>
        </div>

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

.path-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  margin-bottom: 12px;
  background: #f5f5f5;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.2s;
}

.path-bar:hover {
  background: #ebebeb;
}

.path-text {
  font-size: 13px;
  color: #666;
  word-break: break-all;
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
