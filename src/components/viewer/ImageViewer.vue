<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue';
import { NIcon, NText } from 'naive-ui';
import {
  ChevronBackOutline,
  ChevronForwardOutline,
  TrashOutline,
  ExpandOutline,
} from '@vicons/ionicons5';
import { convertFileSrc } from '@tauri-apps/api/core';
import Filmstrip from './Filmstrip.vue';

// ImageInfo interface matching GalleryCards.vue
export interface ImageInfo {
  path: string;
  name: string;
}

// Props
interface Props {
  images: ImageInfo[];
  visible: boolean;
  initialIndex?: number;
}

const props = withDefaults(defineProps<Props>(), {
  initialIndex: 0,
});

// Emits
const emit = defineEmits<{
  'update:visible': [value: boolean];
  delete: [index: number];
}>();

// State
const currentIndex = ref(props.initialIndex || 0);
const zoomLevel = ref(1); // 1 = 100%
const panOffset = ref({ x: 0, y: 0 });
const isDragging = ref(false);
const dragStart = ref({ x: 0, y: 0 });

// Computed
const currentImage = computed(() => props.images[currentIndex.value]);
const canGoPrev = computed(() => currentIndex.value > 0);
const canGoNext = computed(() => currentIndex.value < props.images.length - 1);
const zoomPercent = computed(() => Math.round(zoomLevel.value * 100));
const cursorStyle = computed(() => {
  if (isDragging.value) return 'grabbing';
  if (zoomLevel.value > 1) return 'grab';
  return 'default';
});

// Methods
function getImageSrc(path: string): string {
  return convertFileSrc(path.replace(/\\/g, '/'));
}

function prevImage() {
  if (canGoPrev.value) {
    currentIndex.value--;
    resetZoom();
  }
}

function nextImage() {
  if (canGoNext.value) {
    currentIndex.value++;
    resetZoom();
  }
}

function goToImage(index: number) {
  currentIndex.value = index;
  resetZoom();
}

function zoomIn() {
  zoomLevel.value = Math.min(5, zoomLevel.value + 0.2);
}

function zoomOut() {
  zoomLevel.value = Math.max(0.5, zoomLevel.value - 0.2);
}

function resetZoom() {
  zoomLevel.value = 1;
  panOffset.value = { x: 0, y: 0 };
}

function handleWheel(e: WheelEvent) {
  e.preventDefault();
  if (e.deltaY < 0) {
    zoomIn();
  } else {
    zoomOut();
  }
}

function handleMouseDown(e: MouseEvent) {
  if (zoomLevel.value > 1) {
    isDragging.value = true;
    dragStart.value = { x: e.clientX - panOffset.value.x, y: e.clientY - panOffset.value.y };
  }
}

function handleMouseMove(e: MouseEvent) {
  if (isDragging.value && zoomLevel.value > 1) {
    panOffset.value = {
      x: e.clientX - dragStart.value.x,
      y: e.clientY - dragStart.value.y,
    };
  }
}

function handleMouseUp() {
  isDragging.value = false;
}

function handleKeydown(e: KeyboardEvent) {
  if (!props.visible) return;

  switch (e.key) {
    case 'ArrowLeft':
      e.preventDefault();
      prevImage();
      break;
    case 'ArrowRight':
      e.preventDefault();
      nextImage();
      break;
    case 'Escape':
      e.preventDefault();
      emit('update:visible', false);
      break;
    case '+':
    case '=':
      e.preventDefault();
      zoomIn();
      break;
    case '-':
      e.preventDefault();
      zoomOut();
      break;
    case '0':
      e.preventDefault();
      resetZoom();
      break;
  }
}

function handleOverlayClick(e: MouseEvent) {
  // Close if clicking on the overlay background (not the image container)
  if ((e.target as HTMLElement).classList.contains('viewer-overlay')) {
    emit('update:visible', false);
  }
}

function handleDelete() {
  emit('delete', currentIndex.value);
  // Adjust index if needed after deletion
  if (currentIndex.value >= props.images.length) {
    currentIndex.value = Math.max(0, props.images.length - 1);
  }
}

// Image style for transform
const imageStyle = computed(() => ({
  transform: `scale(${zoomLevel.value}) translate(${panOffset.value.x}px, ${panOffset.value.y}px)`,
  transformOrigin: 'center center',
  cursor: cursorStyle.value,
}));

// Reset zoom when navigating to different image
watch(currentIndex, () => {
  resetZoom();
});

// Event listeners
onMounted(() => {
  window.addEventListener('keydown', handleKeydown);
  window.addEventListener('mouseup', handleMouseUp);
  window.addEventListener('mousemove', handleMouseMove);
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeydown);
  window.removeEventListener('mouseup', handleMouseUp);
  window.removeEventListener('mousemove', handleMouseMove);
});
</script>

<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="viewer-overlay"
      @click="handleOverlayClick"
      @wheel="handleWheel"
    >
      <!-- Close button -->
      <button class="viewer-close" @click="emit('update:visible', false)" aria-label="Close">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"></line>
          <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
      </button>

      <!-- Image container -->
      <div
        class="image-container"
        @mousedown="handleMouseDown"
      >
        <img
          v-if="currentImage"
          :src="getImageSrc(currentImage.path)"
          :alt="currentImage.name"
          :style="imageStyle"
          draggable="false"
        />
      </div>

      <!-- Navigation: Previous -->
      <button
        class="nav-button nav-prev"
        :disabled="!canGoPrev"
        @click.stop="prevImage"
        aria-label="Previous image"
      >
        <n-icon :size="32"><ChevronBackOutline /></n-icon>
      </button>

      <!-- Navigation: Next -->
      <button
        class="nav-button nav-next"
        :disabled="!canGoNext"
        @click.stop="nextImage"
        aria-label="Next image"
      >
        <n-icon :size="32"><ChevronForwardOutline /></n-icon>
      </button>

      <!-- Bottom bar -->
      <div class="viewer-bottom-bar">
        <!-- Counter -->
        <n-text depth="3" class="image-counter">
          {{ currentIndex + 1 }} / {{ images.length }}
        </n-text>

        <!-- Zoom controls -->
        <div class="zoom-controls">
          <button class="zoom-btn" @click.stop="zoomOut" aria-label="Zoom out">-</button>
          <n-text depth="3" class="zoom-level">{{ zoomPercent }}%</n-text>
          <button class="zoom-btn" @click.stop="zoomIn" aria-label="Zoom in">+</button>
        </div>

        <!-- Reset zoom -->
        <button class="reset-zoom-btn" @click.stop="resetZoom" aria-label="Reset zoom">
          <n-icon :size="20"><ExpandOutline /></n-icon>
        </button>

        <!-- Delete button -->
        <button class="delete-btn" @click.stop="handleDelete" aria-label="Delete image">
          <n-icon :size="20"><TrashOutline /></n-icon>
          <n-text>删除</n-text>
        </button>
      </div>

      <!-- Keyboard hints -->
      <div class="keyboard-hints">
        <span>Arrow Left/Right: Navigate</span>
        <span>Scroll: Zoom</span>
        <span>+/-/0: Zoom In/Out/Reset</span>
        <span>Esc: Close</span>
      </div>

      <!-- Filmstrip navigation -->
      <Filmstrip
        :images="props.images"
        :current-index="currentIndex"
        @select="goToImage"
      />
    </div>
  </Teleport>
</template>

<style scoped>
.viewer-overlay {
  position: fixed;
  inset: 0;
  z-index: 9999;
  background: rgba(0, 0, 0, 0.95);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}

.viewer-close {
  position: absolute;
  top: 16px;
  right: 16px;
  background: rgba(255, 255, 255, 0.1);
  border: none;
  border-radius: 50%;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: #fff;
  transition: background 0.2s;
  z-index: 10;
}

.viewer-close:hover {
  background: rgba(255, 255, 255, 0.2);
}

.image-container {
  max-width: 90vw;
  max-height: 80vh;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
}

.image-container img {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
  user-select: none;
  transition: transform 0.1s ease-out;
}

.nav-button {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  background: rgba(255, 255, 255, 0.1);
  border: none;
  border-radius: 50%;
  width: 56px;
  height: 56px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: #fff;
  transition: background 0.2s, opacity 0.2s;
  z-index: 10;
}

.nav-button:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.2);
}

.nav-button:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.nav-prev {
  left: 16px;
}

.nav-next {
  right: 16px;
}

.viewer-bottom-bar {
  position: absolute;
  bottom: 16px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 16px;
  background: rgba(0, 0, 0, 0.8);
  padding: 8px 16px;
  border-radius: 8px;
  z-index: 10;
}

.image-counter {
  font-size: 14px;
  min-width: 60px;
  text-align: center;
}

.zoom-controls {
  display: flex;
  align-items: center;
  gap: 8px;
}

.zoom-btn {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  border: 1px solid rgba(255, 255, 255, 0.2);
  background: transparent;
  color: #fff;
  font-size: 18px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.2s;
}

.zoom-btn:hover {
  background: rgba(255, 255, 255, 0.1);
}

.zoom-level {
  font-size: 14px;
  min-width: 48px;
  text-align: center;
}

.reset-zoom-btn {
  background: transparent;
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 4px;
  padding: 4px 8px;
  color: #fff;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 4px;
  transition: background 0.2s;
}

.reset-zoom-btn:hover {
  background: rgba(255, 255, 255, 0.1);
}

.delete-btn {
  background: transparent;
  border: 1px solid rgba(239, 68, 68, 0.5);
  border-radius: 4px;
  padding: 4px 8px;
  color: #ef4444;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 4px;
  transition: background 0.2s;
}

.delete-btn:hover {
  background: rgba(239, 68, 68, 0.1);
}

.keyboard-hints {
  position: absolute;
  bottom: 80px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  gap: 16px;
  color: rgba(255, 255, 255, 0.4);
  font-size: 12px;
  z-index: 5;
}

.keyboard-hints span {
  white-space: nowrap;
}
</style>