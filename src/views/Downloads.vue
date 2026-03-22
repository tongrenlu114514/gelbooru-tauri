<script setup lang="ts">
import { h } from 'vue'
import {
  NDataTable,
  NButton,
  NSpace,
  NProgress,
  NTag,
  NEmpty
} from 'naive-ui'
import { useDownloadStore } from '@/stores/download'
import type { DataTableColumns } from 'naive-ui'
import type { DownloadTask } from '@/types'

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
    width: 120,
    render(row) {
      const statusMap: Record<string, { type: 'default' | 'info' | 'success' | 'warning' | 'error', label: string }> = {
        pending: { type: 'default', label: '等待中' },
        downloading: { type: 'info', label: '下载中' },
        completed: { type: 'success', label: '已完成' },
        failed: { type: 'error', label: '失败' },
        paused: { type: 'warning', label: '已暂停' }
      }
      const status = statusMap[row.status]
      return h(NTag, { type: status.type, size: 'small' }, { default: () => status.label })
    }
  },
  {
    title: '进度',
    key: 'progress',
    width: 200,
    render(row) {
      if (row.status === 'completed') {
        return h(NProgress, { type: 'line', percentage: 100, status: 'success', showIndicator: false })
      }
      return h(NProgress, { type: 'line', percentage: row.progress, showIndicator: false })
    }
  },
  {
    title: '操作',
    key: 'actions',
    width: 150,
    render(row) {
      return h(NSpace, {}, {
        default: () => [
          row.status === 'downloading' && h(NButton, { size: 'small', onClick: () => pauseTask(row) }, { default: () => '暂停' }),
          row.status === 'paused' && h(NButton, { size: 'small', type: 'primary', onClick: () => resumeTask(row) }, { default: () => '继续' }),
          h(NButton, { size: 'small', type: 'error', onClick: () => downloadStore.removeTask(row.id) }, { default: () => '删除' })
        ]
      })
    }
  }
]

function pauseTask(task: DownloadTask) {
  downloadStore.updateTask(task.id, { status: 'paused' })
}

function resumeTask(task: DownloadTask) {
  downloadStore.updateTask(task.id, { status: 'pending' })
}

function clearCompleted() {
  downloadStore.clearCompleted()
}
</script>

<template>
  <div class="downloads-view">
    <n-space justify="space-between" style="margin-bottom: 16px;">
      <span style="font-size: 18px; font-weight: 500;">下载管理</span>
      <n-space>
        <n-button @click="clearCompleted">清除已完成</n-button>
        <n-button type="primary">开始全部</n-button>
      </n-space>
    </n-space>
    
    <n-data-table
      v-if="downloadStore.tasks.length > 0"
      :columns="columns"
      :data="downloadStore.tasks"
      :bordered="false"
    />
    <n-empty v-else description="暂无下载任务" />
  </div>
</template>

<style scoped>
.downloads-view {
  padding: 0;
}
</style>
