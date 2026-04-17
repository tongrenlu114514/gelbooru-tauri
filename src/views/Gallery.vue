<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, nextTick, h, watch } from 'vue';
import {
  NEmpty,
  NButton,
  NSpace,
  NText,
  NSpin,
  NIcon,
  NTree,
  NModal,
  NLayout,
  NLayoutSider,
  NLayoutContent,
  useMessage,
  useDialog,
  type TreeOption,
} from 'naive-ui';
import {
  RefreshOutline,
  OpenOutline,
  FolderOpenOutline,
  TrashOutline,
  ChevronBackOutline,
  ChevronForwardOutline,
  FolderOutline,
} from '@vicons/ionicons5';
import { invoke } from '@tauri-apps/api/core';
import { convertFileSrc } from '@tauri-apps/api/core';
import { imageBase64Cache } from '../utils/lruCache';
import { useSettingsStore } from '@/stores/settings';

const message = useMessage();
const dialog = useDialog();

// 获取图片 URL（同步，用于模板）
function getImageSrc(path: string): string {
  if (!path) return '';
  // 优先返回缓存的 base64（LRU 缓存自动管理内存），否则返回 asset URL
  const cached = imageBase64Cache.get(path);
  if (cached) {
    return cached;
  }
  return convertFileSrc(path.replace(/\\/g, '/'));
}

// IntersectionObserver ref — null until component mounts
const observerRef = ref<IntersectionObserver | null>(null);

// Re-load the directory tree whenever the download path changes in settings
const settingsStore = useSettingsStore();
watch(
  () => settingsStore.downloadPath,
  () => {
    refresh();
  }
);

// Observe callback: load base64 only for visible images
function observeCallback(entries: IntersectionObserverEntry[]) {
  entries.forEach((entry) => {
    if (entry.isIntersecting) {
      const path = (entry.target as HTMLElement).dataset.imagePath;
      if (path) {
        loadImageBase64(path);
      }
    }
  });
}

// Load image as base64 and cache it (only if not already cached)
async function loadImageBase64(path: string) {
  if (imageBase64Cache.has(path)) return;
  try {
    const base64 = await invoke<string>('get_local_image_base64', { path });
    imageBase64Cache.set(path, base64);
  } catch (err) {
    console.warn('Failed to load image base64:', path, err);
  }
}

// Observe all image cards using IntersectionObserver (viewport-aware lazy loading)
function loadVisibleImages() {
  const grid = document.querySelector('.content-grid');
  if (!grid || !observerRef.value) return;
  const cards = grid.querySelectorAll<HTMLElement>('[data-image-path]');
  cards.forEach((card) => observerRef.value!.observe(card));
}

interface ImageInfo {
  path: string;
  name: string;
}

interface SubDirInfo {
  path: string;
  name: string;
  imageCount: number;
  thumbnail?: string;
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
  if (currentPath.value) {
    try {
      await invoke('open_file', { path: currentPath.value });
    } catch (error) {
      console.error('Failed to open folder:', error);
    }
  }
}

function openPreview(index: number) {
  previewIndex.value = index;
  showPreview.value = true;
}

function prevImage() {
  if (previewIndex.value > 0) {
    previewIndex.value--;
  }
}

function nextImage() {
  if (previewIndex.value < images.value.length - 1) {
    previewIndex.value++;
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (!showPreview.value) return;
  if (e.key === 'ArrowLeft') prevImage();
  if (e.key === 'ArrowRight') nextImage();
  if (e.key === 'Escape') showPreview.value = false;
}

async function openImage(path: string) {
  try {
    await invoke('open_file', { path });
  } catch (error) {
    console.error('Failed to open image:', error);
  }
}

async function openFolder(path: string) {
  try {
    const lastSep = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
    const folderPath = lastSep > 0 ? path.substring(0, lastSep) : path;
    await invoke('open_file', { path: folderPath });
  } catch (error) {
    console.error('Failed to open folder:', error);
  }
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
        if (images.value.length === 0) {
          showPreview.value = false;
        }
        message.success('删除成功');
        // Refresh tree to update counts
        await loadTree();
      } catch (error) {
        message.error(`删除失败: ${error}`);
      }
    },
  });
}

function renderTreeLabel({ option }: { option: TreeOption }) {
  const node = option as unknown as TreeNode;
  return h('div', { class: 'tree-node-label' }, [
    h(
      NIcon,
      {
        size: 16,
        style: { marginRight: '6px' },
        color: '#f0a020',
      },
      () => h(FolderOutline)
    ),
    h('span', { style: { flex: 1 } }, node.label as string),
    h(
      'span',
      {
        style: {
          marginLeft: '8px',
          fontSize: '12px',
          color: '#999',
          backgroundColor: '#f5f5f5',
          padding: '2px 6px',
          borderRadius: '10px',
        },
      },
      `${node.imageCount}`
    ),
  ]);
}

function handleTreeSelect(keys: string[]) {
  if (keys.length > 0) {
    selectedKey.value = keys[0];
    const node = findNodeByKey(treeData.value, keys[0]);
    if (node) {
      loadImagesForDirectory(node.path);
    } else {
      images.value = [];
    }
  }
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

async function loadTree() {
  loadingTree.value = true;
  try {
    const result = await invoke<TreeNode[]>('get_directory_tree', {});
    treeData.value = result;
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
      {
        dirPath,
      }
    );
    subdirs.value = result.subdirs;
    images.value = result.images;

    // Observe newly rendered image cards for viewport-based lazy loading
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

// 处理图片加载错误，尝试使用 base64（使用 LRU 缓存防止内存泄漏）
function handleImageError(event: Event, path: string) {
  const img = event.target as HTMLImageElement;
  if (img && path && !imageBase64Cache.has(path)) {
    // 加载 base64 作为后备
    invoke<string>('get_local_image_base64', { path })
      .then((base64) => {
        imageBase64Cache.set(path, base64);
        img.src = base64;
      })
      .catch((err) => console.error('Failed to load base64 fallback:', err));
  }
}

function enterSubdir(subdir: SubDirInfo) {
  selectedKey.value = subdir.path;
  loadImagesForDirectory(subdir.path);
  // Expand tree node
  const node = findNodeByKey(treeData.value, subdir.path);
  if (node) {
    // Tree will auto-expand when selected
  }
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

// Expose loadVisibleImages for testing via template ref / wrapper.vm
defineExpose({ loadVisibleImages });
</script>

<template>
  <div class="gallery-view">
    <n-space justify="space-between" align="center" style="margin-bottom: 16px">
      <span style="font-size: 18px; font-weight: 500"> 本地图库 </span>
      <n-button @click="refresh" :loading="loadingTree">
        <template #icon>
          <n-icon><RefreshOutline /></n-icon>
        </template>
        刷新
      </n-button>
    </n-space>

    <n-layout has-sider style="height: calc(100vh - 140px)">
      <n-layout-sider bordered :width="280" :native-scrollbar="false" content-style="padding: 8px;">
        <n-spin :show="loadingTree">
          <n-tree
            v-if="treeData.length > 0"
            :data="treeData"
            :render-label="renderTreeLabel"
            :selected-keys="selectedKey ? [selectedKey] : []"
            block-line
            expand-on-click
            @update:selected-keys="handleTreeSelect"
          />
          <n-empty v-else description="暂无本地图片" />
        </n-spin>
      </n-layout-sider>

      <n-layout-content content-style="padding: 12px;">
        <div v-if="selectedKey" class="path-bar" @click="openCurrentFolder">
          <n-icon :size="16"><FolderOpenOutline /></n-icon>
          <span class="path-text">{{ currentPath }}</span>
        </div>
        <n-spin :show="loadingImages">
          <div v-if="subdirs.length > 0 || images.length > 0" class="content-grid">
            <!-- 子目录卡片 -->
            <div
              v-for="subdir in subdirs"
              :key="subdir.path"
              class="folder-card"
              :data-image-path="subdir.thumbnail"
              @click="enterSubdir(subdir)"
            >
              <div class="folder-preview">
                <img
                  v-if="subdir.thumbnail"
                  :src="getImageSrc(subdir.thumbnail)"
                  alt=""
                  @error="handleImageError($event, subdir.thumbnail)"
                />
                <n-icon v-else :size="48" color="#f0a020"><FolderOutline /></n-icon>
              </div>
              <div class="folder-info">
                <n-icon :size="16" color="#f0a020"><FolderOutline /></n-icon>
                <span class="folder-name">{{ subdir.name }}</span>
                <span class="folder-count">{{ subdir.imageCount }}</span>
              </div>
            </div>
            <!-- 图片卡片 -->
            <div
              v-for="(img, index) in images"
              :key="img.path"
              class="image-card"
              :data-image-path="img.path"
              @click="openPreview(index)"
            >
              <img
                :src="getImageSrc(img.path)"
                :alt="img.name"
                @error="handleImageError($event, img.path)"
              />
              <div class="image-overlay">
                <div class="image-name">{{ img.name }}</div>
                <div class="image-actions">
                  <n-button size="tiny" quaternary @click.stop="openImage(img.path)">
                    <template #icon>
                      <n-icon><OpenOutline /></n-icon>
                    </template>
                  </n-button>
                  <n-button size="tiny" quaternary @click.stop="openFolder(img.path)">
                    <template #icon>
                      <n-icon><FolderOpenOutline /></n-icon>
                    </template>
                  </n-button>
                  <n-button size="tiny" quaternary @click.stop="deleteImage(index)">
                    <template #icon>
                      <n-icon><TrashOutline /></n-icon>
                    </template>
                  </n-button>
                </div>
              </div>
            </div>
          </div>
          <n-empty v-else-if="!loadingImages && selectedKey" description="该目录下没有图片" />
          <n-empty v-else-if="!loadingImages" description="请从左侧选择目录查看图片" />
        </n-spin>
      </n-layout-content>
    </n-layout>

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

.tree-node-label {
  display: flex;
  align-items: center;
  width: 100%;
  padding: 2px 0;
}

.content-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 12px;
}

.folder-card {
  position: relative;
  aspect-ratio: 1;
  border-radius: 8px;
  overflow: hidden;
  cursor: pointer;
  background: linear-gradient(135deg, #fff8e6 0%, #fff3d6 100%);
  border: 2px solid #f0a020;
  box-shadow: 0 2px 8px rgba(240, 160, 32, 0.2);
  transition:
    transform 0.2s,
    box-shadow 0.2s;
}

.folder-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(240, 160, 32, 0.35);
}

.folder-preview {
  width: 100%;
  height: calc(100% - 36px);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  background: rgba(0, 0, 0, 0.03);
}

.folder-preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  opacity: 0.85;
}

.folder-info {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  background: rgba(240, 160, 32, 0.15);
}

.folder-name {
  flex: 1;
  font-size: 12px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: #8b6914;
}

.folder-count {
  font-size: 11px;
  color: #a08020;
  background: rgba(255, 255, 255, 0.7);
  padding: 2px 6px;
  border-radius: 8px;
}

.image-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 12px;
}

.image-card {
  position: relative;
  aspect-ratio: 1;
  border-radius: 8px;
  overflow: hidden;
  cursor: pointer;
  background: #f5f5f5;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  transition:
    transform 0.2s,
    box-shadow 0.2s;
}

.image-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.image-card img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.image-overlay {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  background: linear-gradient(transparent, rgba(0, 0, 0, 0.7));
  padding: 24px 8px 8px;
  opacity: 0;
  transition: opacity 0.2s;
}

.image-card:hover .image-overlay {
  opacity: 1;
}

.image-name {
  color: white;
  font-size: 12px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.image-actions {
  display: flex;
  gap: 4px;
  margin-top: 4px;
}

.image-actions .n-button {
  color: white;
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
