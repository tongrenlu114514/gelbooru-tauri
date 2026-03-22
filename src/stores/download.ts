import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useSettingsStore } from './settings'

export interface DownloadTask {
  id: number
  postId: number
  imageUrl: string
  fileName: string
  savePath: string
  status: 'pending' | 'downloading' | 'completed' | 'failed' | 'paused' | 'cancelled'
  progress: number
  downloadedSize: number
  totalSize: number
  error?: string
}

interface DownloadProgressEvent {
  id: number
  postId: number
  status: string
  progress: number
  downloadedSize: number
  totalSize: number
  error?: string
}

export const useDownloadStore = defineStore('download', () => {
  const tasks = ref<DownloadTask[]>([])
  const isDownloading = ref(false)
  let unlistenProgress: UnlistenFn | null = null
  
  const queue = computed(() => tasks.value.filter(t => t.status === 'pending'))
  const downloading = computed(() => tasks.value.filter(t => t.status === 'downloading'))
  const completed = computed(() => tasks.value.filter(t => t.status === 'completed'))
  const failed = computed(() => tasks.value.filter(t => t.status === 'failed'))
  
  // 初始化事件监听
  async function initListeners() {
    if (unlistenProgress) return
    
    // 监听下载进度
    unlistenProgress = await listen<DownloadProgressEvent>('download-progress', (event) => {
      const data = event.payload
      const index = tasks.value.findIndex(t => t.id === data.id)
      if (index !== -1) {
        tasks.value[index] = {
          ...tasks.value[index],
          status: data.status as DownloadTask['status'],
          progress: data.progress,
          downloadedSize: data.downloadedSize,
          totalSize: data.totalSize,
          error: data.error
        }
      }
    })
  }
  
  // 添加任务
  async function addTask(options: {
    postId: number
    imageUrl: string
    fileName: string
  }) {
    await initListeners()
    
    const settingsStore = useSettingsStore()
    const savePath = `${settingsStore.downloadPath}/${options.fileName}`
    
    try {
      const task = await invoke<DownloadTask>('add_download_task', {
        postId: options.postId,
        imageUrl: options.imageUrl,
        fileName: options.fileName,
        savePath: savePath
      })
      
      tasks.value.push(task)
      
      // 自动开始下载
      await startDownload(task.id)
      
      return task
    } catch (error) {
      console.error('Failed to add download task:', error)
      throw error
    }
  }
  
  // 开始下载
  async function startDownload(id: number) {
    try {
      await invoke('start_download', { id })
      isDownloading.value = true
    } catch (error) {
      console.error('Failed to start download:', error)
      throw error
    }
  }
  
  // 暂停下载
  async function pauseDownload(id: number) {
    try {
      await invoke('pause_download', { id })
    } catch (error) {
      console.error('Failed to pause download:', error)
    }
  }
  
  // 恢复下载
  async function resumeDownload(id: number) {
    try {
      await invoke('resume_download', { id })
    } catch (error) {
      console.error('Failed to resume download:', error)
    }
  }
  
  // 取消下载
  async function cancelDownload(id: number) {
    try {
      await invoke('cancel_download', { id })
    } catch (error) {
      console.error('Failed to cancel download:', error)
    }
  }
  
  // 移除任务
  async function removeTask(id: number) {
    try {
      await invoke('remove_download_task', { id })
      const index = tasks.value.findIndex(t => t.id === id)
      if (index !== -1) {
        tasks.value.splice(index, 1)
      }
    } catch (error) {
      console.error('Failed to remove task:', error)
    }
  }
  
  // 清除已完成
  async function clearCompleted() {
    try {
      await invoke('clear_completed_tasks')
      tasks.value = tasks.value.filter(t => t.status !== 'completed')
    } catch (error) {
      console.error('Failed to clear completed tasks:', error)
    }
  }
  
  // 获取所有任务
  async function fetchTasks() {
    try {
      const result = await invoke<DownloadTask[]>('get_download_tasks')
      tasks.value = result
    } catch (error) {
      console.error('Failed to fetch tasks:', error)
    }
  }
  
  // 开始所有待下载任务
  async function startAllPending() {
    for (const task of queue.value) {
      await startDownload(task.id)
    }
  }
  
  // 暂停所有下载
  async function pauseAllDownloading() {
    for (const task of downloading.value) {
      await pauseDownload(task.id)
    }
  }
  
  // 打开文件
  async function openFile(path: string) {
    try {
      await invoke('open_file', { path })
    } catch (error) {
      console.error('Failed to open file:', error)
    }
  }

  return {
    tasks,
    isDownloading,
    queue,
    downloading,
    completed,
    failed,
    initListeners,
    addTask,
    startDownload,
    pauseDownload,
    resumeDownload,
    cancelDownload,
    removeTask,
    clearCompleted,
    fetchTasks,
    startAllPending,
    pauseAllDownloading,
    openFile
  }
})