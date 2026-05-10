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
import GalleryCards from './GalleryCards.vue';
import type { ImageInfo, SubDirInfo } from './GalleryCards.vue';
import { useSettingsStore } from '@/stores/settings';

const message = useMessage();
const dialog = useDialog();
const settingsStore = useSettingsStore();

const galleryCardsRef = ref<InstanceType<typeof GalleryCards> | null>(null);

// Preview modal: use convertFileSrc for full-size image
function getPreviewSrc(path: string): string {
  return convertFileSrc(path.replace(/\\/g, '/'));
}

// IntersectionObserver — lazy load images as they enter the viewport
const observerRef = ref<IntersectionObserver | null>(null);

// Infinite scroll observer for "load more" sentinel
const loadMoreObserverRef = ref<IntersectionObserver | null>(null);

function observeCallback(entries: IntersectionObserverEntry[]) {
  entries.forEach((entry) => {
    if (!entry.isIntersecting) return;
    const card = entry.target as HTMLElement;
    const path = card.dataset.imagePath;
    if (!path) return;
    const src = convertFileSrc(path.replace(/\\/g, '/'));
    // Try galleryCardsRef first (works in real browser)
    if (galleryCardsRef.value && typeof galleryCardsRef.value.setCardSrc === 'function') {
      galleryCardsRef.value.setCardSrc(path, src);
    } else {
      // Fallback: use data attribute to locate GalleryCards root element,
      // then call setCardSrc via __vueParentComponent.exposed
      const card = entry.target as HTMLElement;
      const gc = card.closest('[data-gallery-cards]') as any;
      if (gc) {
        const exposed = gc.__vueParentComponent?.exposed;
        if (exposed?.setCardSrc) {
          exposed.setCardSrc(path, src);
        }
      }
    }
    observerRef.value?.unobserve(card); // loaded — stop watching
  });
}

function loadVisibleImages() {
  const grid = document.querySelector('.content-grid');
  if (!grid || !observerRef.value) return;
  const cards = grid.querySelectorAll<HTMLElement>('[data-image-path]');
  cards.forEach((card) => observerRef.value!.observe(card));
}

/** Wait for MasonryWall to finish populating [data-image-path] cards in the DOM */
function waitForCards(timeoutMs: number): Promise<void> {
  return new Promise((resolve) => {
    if (document.querySelector('[data-image-path]')) {
      resolve();
      return;
    }
    const observer = new MutationObserver(() => {
      if (document.querySelector('[data-image-path]')) {
        observer.disconnect();
        resolve();
      }
    });
    const start = Date.now();
    const poll = setInterval(() => {
      if (document.querySelector('[data-image-path]')) {
        clearInterval(poll);
        observer.disconnect();
        resolve();
        return;
      }
      if (Date.now() - start > timeoutMs) {
        clearInterval(poll);
        observer.disconnect();
        resolve(); // don't block, just proceed
      }
    }, 50);
  });
}

const selectedKey = ref<string | null>(null);
const images = ref<ImageInfo[]>([]);
const loadingTree = ref(false);
const loadingImages = ref(false);

// Pagination state
const page = ref(0);
const limit = ref(50);
const hasMore = ref(true);

// Sentinel ref for infinite scroll trigger
const loadMoreRef = ref<HTMLElement | null>(null);

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

// Navigate to root (downloadPath) via root breadcrumb segment
function goToRoot() {
  if (!settingsStore.downloadPath) return;
  if (selectedKey.value === settingsStore.downloadPath) return; // Already at root — no-op
  const rootSubdir: SubDirInfo = {
    path: settingsStore.downloadPath,
    name: '根目录',
    imageCount: 0,
  };
  enterSubdir(rootSubdir);
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

async function loadImagesForDirectory(dirPath: string, reset = true) {
  if (reset) {
    loadingImages.value = true;
    page.value = 0;
    images.value = [];
  } else {
    loadingImages.value = true;
  }
  try {
    const result = await invoke<{
      subdirs: SubDirInfo[];
      images: ImageInfo[];
      total: number;
      has_more: boolean;
      offset: number;
      limit: number;
    }>('get_directory_images', {
      dirPath,
      page: page.value,
      limit: limit.value,
    });
    if (reset) {
      images.value = result.images;
    } else {
      images.value.push(...result.images);
    }
    hasMore.value = result.has_more;
    if (reset) {
      // Wait for MasonryWall async redraw() to finish — fillColumns is deeply
      // recursive (N items × N nextTicks). Use MutationObserver to detect
      // when [data-image-path] cards are actually in the DOM, then observe them.
      if (!observerRef.value) {
        observerRef.value = new IntersectionObserver(observeCallback, {
          root: null,
          rootMargin: '200px',
          threshold: 0.01,
        });
      }
      await waitForCards(5000);
      loadVisibleImages();
      setupLoadMoreObserver();
    }
  } catch (error) {
    console.error('Failed to load images:', error);
    if (reset) images.value = [];
  } finally {
    loadingImages.value = false;
  }
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

// Infinite scroll: sentinel enters viewport → load next page
function setupLoadMoreObserver() {
  const sentinel = loadMoreRef.value;
  if (!sentinel || loadMoreObserverRef.value) return;
  loadMoreObserverRef.value = new IntersectionObserver((entries) => {
    if (entries[0].isIntersecting && hasMore.value && !loadingImages.value) {
      page.value++;
      loadImagesForDirectory(selectedKey.value!, false);
    }
  }, { rootMargin: '200px', threshold: 0.01 });
  loadMoreObserverRef.value.observe(sentinel);
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

async function refresh() {
  observerRef.value?.disconnect();
  loadMoreObserverRef.value?.disconnect();
  observerRef.value = null;
  loadMoreObserverRef.value = null;
  selectedKey.value = null;
  images.value = [];
  page.value = 0;
  hasMore.value = true;
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
});

onUnmounted(() => {
  observerRef.value?.disconnect();
  loadMoreObserverRef.value?.disconnect();
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
        <div v-if="breadcrumbSegments.length > 0 || selectedKey !== null" class="breadcrumb-bar">
          <n-breadcrumb>
            <!-- Root segment: always clickable to go to downloadPath -->
            <n-breadcrumb-item
              clickable
              @click="goToRoot"
            >
              <n-icon :size="14" color="#f0a020" style="margin-right: 4px">
                <FolderOpenOutline />
              </n-icon>
              根目录
            </n-breadcrumb-item>
            <!-- Child segments: only clickable if not last (not current folder) -->
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

        <!-- Image card grid + infinite scroll sentinel -->
        <GalleryCards
          ref="galleryCardsRef"
          :images="images"
          :loading-images="loadingImages"
          :selected-key="selectedKey"
          @open-preview="openPreview"
        />
        <!-- Infinite scroll: sentinel below masonry triggers next page load -->
        <div ref="loadMoreRef" class="load-more-sentinel">
          <n-spin v-if="loadingImages" size="small" />
        </div>
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
          :src="getPreviewSrc(currentImage.path)"
          style="max-width: 80vw; max-height: 70vh; object-fit: contain"
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

/* Infinite scroll sentinel: thin div below masonry grid */
.load-more-sentinel {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 16px;
  min-height: 40px;
}

.preview-container {
  display: flex;
  align-items: center;
  gap: 16px;
}
</style>