import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useSettingsStore } from './settings'
import type { GelbooruTag } from '@/types'

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

interface PostMeta {
  postId: number
  imageUrl: string
  posted: string      // 日期
  rating: string      // 分级
  tags: GelbooruTag[] // 标签列表
}

// 清理文件名中的非法字符
function sanitizeFileName(name: string): string {
  return name
    .replace(/[<>:"/\\|?*]/g, '_')
    .replace(/\s+/g, '_')
    .replace(/_{2,}/g, '_')
    .replace(/^_|_$/g, '')
}

// 从标签列表中提取特定类型的标签
function extractTagsByType(tags: GelbooruTag[], type: string): string[] {
  return tags
    .filter(t => t.tagType.toLowerCase() === type.toLowerCase())
    .map(t => sanitizeFileName(t.text))
}

// 生成保存路径: {日期}/{分级}/{作品}/[{角色}]{id}(artist).{ext}
function generateSavePath(meta: PostMeta, basePath: string): string {
  // 提取各类型标签
  const artists = extractTagsByType(meta.tags, 'artist')
  const characters = extractTagsByType(meta.tags, 'character')
  const copyrights = extractTagsByType(meta.tags, 'copyright')
  
  // 解析日期 (格式可能是 "2024-03-22 12:34:56" 或其他)
  let dateStr = 'unknown'
  if (meta.posted) {
    const match = meta.posted.match(/^(\d{4}-\d{2}-\d{2})/)
    if (match) {
      dateStr = match[1]
    }
  }
  
  // 分级
  let rating = meta.rating?.toLowerCase() || 'unknown'
  if (rating === 'safe' || rating === 's') rating = 'safe'
  else if (rating === 'questionable' || rating === 'q') rating = 'questionable'
  else if (rating === 'explicit' || rating === 'e') rating = 'explicit'
  
  // 作品/版权（取第一个）
  const copyright = copyrights.length > 0 ? copyrights[0] : 'unknown'
  
  // 角色列表（用逗号连接）
  const characterPart = characters.length > 0 ? `[${characters.join(',')}]` : ''
  
  // 艺术家（取第一个）
  const artistPart = artists.length > 0 ? `(${artists[0]})` : ''
  
  // 获取扩展名
  const ext = meta.imageUrl.split('.').pop()?.split('?')[0] || 'jpg'
  
  // 构建文件名: [角色]id(艺术家).ext
  const fileName = `${characterPart}${meta.postId}${artistPart}.${ext}`
  
  // 构建完整路径: 基础路径/日期/分级/作品/文件名
  const savePath = `${basePath}/${dateStr}/${rating}/${copyright}/${fileName}`
  
  return savePath
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
  async function addTask(meta: PostMeta) {
    await initListeners()
    
    const settingsStore = useSettingsStore()
    const savePath = generateSavePath(meta, settingsStore.downloadPath)
    
    // 提取文件名用于显示
    const fileName = savePath.split('/').pop() || `${meta.postId}`
    
    try {
      const task = await invoke<DownloadTask>('add_download_task', {
        postId: meta.postId,
        imageUrl: meta.imageUrl,
        fileName: fileName,
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