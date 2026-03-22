import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useSettingsStore = defineStore('settings', () => {
  const theme = ref<'light' | 'dark'>('dark')
  const sidebarCollapsed = ref(false)
  const downloadPath = ref('D:/project/gelbooru/imgs/')
  const concurrentDownloads = ref(3)
  
  function toggleTheme() {
    theme.value = theme.value === 'dark' ? 'light' : 'dark'
  }
  
  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value
  }

  return {
    theme,
    sidebarCollapsed,
    downloadPath,
    concurrentDownloads,
    toggleTheme,
    toggleSidebar
  }
})
