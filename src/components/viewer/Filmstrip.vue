<script setup lang="ts">
import { computed, ref, watch, nextTick } from 'vue';
import { convertFileSrc } from '@tauri-apps/api/core';

export interface ImageInfo {
  path: string;
  name: string;
}

// Props
interface Props {
  images: ImageInfo[];
  currentIndex: number;
}

const props = withDefaults(defineProps<Props>(), {
  images: () => [],
  currentIndex: 0,
});

// Emits
const emit = defineEmits<{
  select: [index: number];
}>();

// Refs
const trackRef = ref<HTMLElement | null>(null);

// Computed: show 9 thumbnails (4 before, current, 4 after) centered on currentIndex
const halfCount = Math.floor(9 / 2); // 4

const visibleThumbnails = computed(() => {
  const start = Math.max(0, props.currentIndex - halfCount);
  const end = Math.min(props.images.length - 1, props.currentIndex + halfCount);
  const items: Array<{ index: number; image: ImageInfo }> = [];
  for (let i = start; i <= end; i++) {
    items.push({ index: i, image: props.images[i] });
  }
  return items;
});

function thumbSrc(path: string): string {
  return convertFileSrc(path.replace(/\\/g, '/'));
}

function handleSelect(index: number) {
  emit('select', index);
}

// Auto-scroll: center the active thumbnail when currentIndex changes
watch(
  () => props.currentIndex,
  async () => {
    await nextTick();
    if (!trackRef.value) return;
    const track = trackRef.value;
    const activeEl = track.querySelector('.filmstrip-thumb.active') as HTMLElement;
    if (!activeEl) return;
    const trackRect = track.getBoundingClientRect();
    const elRect = activeEl.getBoundingClientRect();
    const scrollTarget = activeEl.offsetLeft - trackRect.width / 2 + elRect.width / 2;
    track.scrollTo({ left: Math.max(0, scrollTarget), behavior: 'smooth' });
  },
  { immediate: true }
);
</script>

<template>
  <div class="filmstrip" @click.self>
    <div ref="trackRef" class="filmstrip-track">
      <div
        v-for="item in visibleThumbnails"
        :key="item.index"
        class="filmstrip-thumb"
        :class="{ active: item.index === currentIndex }"
        @click="handleSelect(item.index)"
      >
        <img :src="thumbSrc(item.image.path)" :alt="item.image.name" loading="lazy" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.filmstrip {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  height: 88px;
  background: rgba(0, 0, 0, 0.85);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 20;
  padding: 0 16px;
}

.filmstrip-track {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 8px;
  overflow-x: auto;
  max-width: 100%;
  scrollbar-width: thin;
  scrollbar-color: rgba(255, 255, 255, 0.3) transparent;
  padding: 6px 0;
}

.filmstrip-track::-webkit-scrollbar {
  height: 4px;
}

.filmstrip-track::-webkit-scrollbar-track {
  background: transparent;
}

.filmstrip-track::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.3);
  border-radius: 2px;
}

.filmstrip-thumb {
  flex: 0 0 auto;
  width: 60px;
  height: 72px;
  border-radius: 4px;
  overflow: hidden;
  border: 2px solid transparent;
  cursor: pointer;
  transition: border-color 0.15s, transform 0.15s, opacity 0.15s;
  background: rgba(255, 255, 255, 0.05);
}

.filmstrip-thumb:hover {
  transform: scale(1.05);
  border-color: rgba(255, 255, 255, 0.4);
}

.filmstrip-thumb.active {
  border-color: rgba(255, 255, 255, 0.9);
  opacity: 1;
}

.filmstrip-thumb:not(.active) {
  opacity: 0.6;
}

.filmstrip-thumb img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}
</style>