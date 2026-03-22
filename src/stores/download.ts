import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { DownloadTask } from '@/types'

export const useDownloadStore = defineStore('download', () => {
  const tasks = ref<DownloadTask[]>([])
  const isDownloading = ref(false)
  
  const queue = computed(() => tasks.value.filter(t => t.status === 'pending'))
  const downloading = computed(() => tasks.value.filter(t => t.status === 'downloading'))
  const completed = computed(() => tasks.value.filter(t => t.status === 'completed'))
  const failed = computed(() => tasks.value.filter(t => t.status === 'failed'))
  
  function addTask(task: DownloadTask) {
    tasks.value.push(task)
  }
  
  function updateTask(id: number, updates: Partial<DownloadTask>) {
    const index = tasks.value.findIndex(t => t.id === id)
    if (index !== -1) {
      tasks.value[index] = { ...tasks.value[index], ...updates }
    }
  }
  
  function removeTask(id: number) {
    const index = tasks.value.findIndex(t => t.id === id)
    if (index !== -1) {
      tasks.value.splice(index, 1)
    }
  }
  
  function clearCompleted() {
    tasks.value = tasks.value.filter(t => t.status !== 'completed')
  }

  return {
    tasks,
    isDownloading,
    queue,
    downloading,
    completed,
    failed,
    addTask,
    updateTask,
    removeTask,
    clearCompleted
  }
})
