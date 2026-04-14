<script setup lang="ts">
import { useNotification } from 'naive-ui';
import { onMounted, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useDownloadStore } from '@/stores/download';

const notification = useNotification();
const downloadStore = useDownloadStore();

let unlistenProgress: UnlistenFn | null = null;

onMounted(async () => {
  // 监听下载进度事件，显示通知
  unlistenProgress = await listen<{
    id: number;
    postId: number;
    status: string;
    progress: number;
    downloadedSize: number;
    totalSize: number;
    error?: string;
  }>('download-progress', (event) => {
    const data = event.payload;

    // 下载完成时弹出通知
    if (data.status === 'completed') {
      const task = downloadStore.tasks.find((t) => t.id === data.id);
      notification.success({
        title: '下载完成',
        content: task?.fileName || `Post #${data.postId}`,
        duration: 3000,
      });
    }

    // 下载失败时弹出通知
    if (data.status === 'failed') {
      const task = downloadStore.tasks.find((t) => t.id === data.id);
      notification.error({
        title: '下载失败',
        content: task?.fileName || `Post #${data.postId}`,
        description: data.error,
        duration: 5000,
      });
    }
  });
});

onUnmounted(() => {
  if (unlistenProgress) {
    unlistenProgress();
  }
});
</script>

<template>
  <!-- 无渲染组件 -->
</template>
