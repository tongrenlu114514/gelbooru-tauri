<script setup lang="ts">
import { h, onMounted, type VNode } from 'vue'
import {
  NDataTable,
  NButton,
  NSpace,
  NProgress,
  NTag,
  NEmpty,
  NPopconfirm
} from 'naive-ui'
import { useDownloadStore } from '@/stores/download'
import type { DataTableColumns } from 'naive-ui'
import type { DownloadTask } from '@/stores/download'

const downloadStore = useDownloadStore()

const columns: DataTableColumns<DownloadTask> = [
  {
    title: 'ID',
    key: 'postId',
    width: 80
  },
  {
    title: '文件名',
    key: 'fileName',
    ellipsis: { tooltip: true }
  },
  {
    title: '状态',
    key: 'status',
    width: 100,
    render(row) {
      const statusMap: Record<string, { type: 'default' | 'info' | 'success' | 'warning' | 'error', label: string }> = {
        pending: { type: 'default', label: '等待中' },
        downloading: { type: 'info', label: '下载中' },
        completed: { type: 'success', label: '已完成' },
        failed: { type: 'error', label: '失败' },
        paused: { type: 'warning', label: '已暂停' },
        cancelled: { type: 'default', label: '已取消' }
      }
      const status = statusMap[row.status] || { type: 'default', label: row.status }
      return h(NTag, { type: status.type, size: 'small' }, { default: () => status.label })
    }
  },
  {
    title: '进度',
    key: 'progress',
    width: 200,
    render(row) {
      const status = row.status === 'completed' ? 'success' : 
                     row.status === 'failed' ? 'error' : 
                     row.status === 'paused' ? 'warning' : 'default'
      const percentage = Math.round(row.progress)
      return h(NProgress, { 
        type: 'line', 
        percentage, 
        status,
        showIndicator: true,
        indicatorPlacement: 'inside'
      })
    }
  },
  {
    title: '大小',
    key: 'size',
    width: 120,
    render(row) {
      if (row.totalSize > 0) {
        return `${formatSize(row.downloadedSize)} / ${formatSize(row.totalSize)}`
      }
      return formatSize(row.downloadedSize)
    }
  },
  {
    title: '操作',
    key: 'actions',
    width: 180,
    render(row) {
      const buttons: VNode[] = []
      
      if (row.status === 'pending') {
        buttons.push(
          h(NButton, { size: 'small', type: 'primary', onClick: () => downloadStore.startDownload(row.id) }, 
            { default: () => '开始' })
        )
      }
      
      if (row.status === 'downloading') {
        buttons.push(
          h(NButton, { size: 'small', onClick: () => downloadStore.pauseDownload(row.id) }, 
            { default: () => '暂停' })
        )
      }
      
      if (row.status === 'paused') {
        buttons.push(
          h(NButton, { size: 'small', type: 'primary', onClick: () => downloadStore.resumeDownload(row.id) }, 
            { default: () => '继续' })
        )
      }
      
      if (row.status === 'failed') {
        buttons.push(
          h(NButton, { size: 'small', type: 'primary', onClick: () => downloadStore.startDownload(row.id) }, 
            { default: () => '重试' })
        )
      }
      
      if (row.status === 'completed') {
        buttons.push(
          h(NButton, { size: 'small', type: 'success', onClick: () => downloadStore.openFile(row.savePath) }, 
            { default: () => '打开' })
        )
      }
      
      const removeBtn = h(
        NPopconfirm,
        { onPositiveClick: () => downloadStore.removeTask(row.id) },
        {
          trigger: () => h(NButton, { size: 'small', type: 'error' }, { default: () => '删除' }),
          default: () => '确定删除此任务？'
        }
      )
      buttons.push(removeBtn)
      
      return h(NSpace, { size: 'small' }, { default: () => buttons })
    }
  }
]

function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
}

async function startAll() {
  await downloadStore.startAllPending()
}

async function pauseAll() {
  await downloadStore.pauseAllDownloading()
}

async function clearCompleted() {
  await downloadStore.clearCompleted()
}

onMounted(() => {
  downloadStore.initListeners()
  downloadStore.fetchTasks()
})
</script>

<template>
  <div class="downloads-view">
    <n-space justify="space-between" style="margin-bottom: 16px;">
      <span style="font-size: 18px; font-weight: 500;">下载管理</span>
      <n-space>
        <n-button @click="startAll" :disabled="downloadStore.queue.length === 0">
          开始全部
        </n-button>
        <n-button @click="pauseAll" :disabled="downloadStore.downloading.length === 0">
          暂停全部
        </n-button>
        <n-button @click="clearCompleted" :disabled="downloadStore.completed.length === 0">
          清除已完成
        </n-button>
      </n-space>
    </n-space>
    
    <div class="stats-bar" v-if="downloadStore.tasks.length > 0">
      <n-space size="large">
        <span>总计: {{ downloadStore.tasks.length }}</span>
        <span style="color: #18a058;">已完成: {{ downloadStore.completed.length }}</span>
        <span style="color: #2080f0;">下载中: {{ downloadStore.downloading.length }}</span>
        <span style="color: #f0a020;">等待中: {{ downloadStore.queue.length }}</span>
        <span style="color: #d03050;">失败: {{ downloadStore.failed.length }}</span>
      </n-space>
    </div>
    
    <n-data-table
      v-if="downloadStore.tasks.length > 0"
      :columns="columns"
      :data="downloadStore.tasks"
      :bordered="false"
      :row-key="(row: DownloadTask) => row.id"
    />
    <n-empty v-else description="暂无下载任务" style="margin-top: 100px;" />
  </div>
</template>

<style scoped>
.downloads-view {
  padding: 0;
}

.stats-bar {
  padding: 12px 16px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 8px;
  margin-bottom: 16px;
  font-size: 13px;
}
</style>