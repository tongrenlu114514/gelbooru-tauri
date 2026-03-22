import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useSettingsStore = defineStore('settings', () => {
  const theme = ref<'light' | 'dark'>('dark')
  const sidebarCollapsed = ref(false)
  const downloadPath = ref('D:/project/gelbooru/imgs/')
  const concurrentDownloads = ref(3)
  const proxyEnabled = ref(true)
  const proxyHost = ref('127.0.0.1')
  const proxyPort = ref(7897)
  
  function toggleTheme() {
    theme.value = theme.value === 'dark' ? 'light' : 'dark'
  }
  
  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value
  }
  
  function getProxyUrl(): string {
    if (!proxyEnabled.value) return ''
    return `http://${proxyHost.value}:${proxyPort.value}`
  }

  return {
    theme,
    sidebarCollapsed,
    downloadPath,
    concurrentDownloads,
    proxyEnabled,
    proxyHost,
    proxyPort,
    toggleTheme,
    toggleSidebar,
    getProxyUrl
  }
})
