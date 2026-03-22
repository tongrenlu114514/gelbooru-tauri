<script setup lang="ts">
import {
  NForm,
  NFormItem,
  NInput,
  NInputNumber,
  NButton,
  NSpace,
  NSwitch,
  NDivider,
  NGrid,
  NGi
} from 'naive-ui'
import { useSettingsStore } from '@/stores/settings'
import { storeToRefs } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

const settingsStore = useSettingsStore()
const { theme, downloadPath, concurrentDownloads, proxyEnabled, proxyHost, proxyPort } = storeToRefs(settingsStore)

async function saveSettings() {
  // Save settings to localStorage
  localStorage.setItem('settings', JSON.stringify({
    theme: theme.value,
    downloadPath: downloadPath.value,
    concurrentDownloads: concurrentDownloads.value,
    proxyEnabled: proxyEnabled.value,
    proxyHost: proxyHost.value,
    proxyPort: proxyPort.value
  }))
  
  // 更新后端代理设置
  const proxyUrl = settingsStore.getProxyUrl()
  try {
    await invoke('set_proxy', { proxyUrl })
  } catch (error) {
    console.error('Failed to set proxy:', error)
  }
}
</script>

<template>
  <div class="settings-view">
    <span style="font-size: 18px; font-weight: 500;">设置</span>
    
    <n-form label-placement="left" label-width="120px" style="margin-top: 20px; max-width: 500px;">
      <n-form-item label="深色模式">
        <n-switch
          :value="theme === 'dark'"
          @update:value="settingsStore.toggleTheme"
        />
      </n-form-item>
      
      <n-divider />
      
      <n-form-item label="下载路径">
        <n-input v-model:value="downloadPath" placeholder="下载保存路径" />
      </n-form-item>
      
      <n-form-item label="并发下载数">
        <n-input-number
          v-model:value="concurrentDownloads"
          :min="1"
          :max="10"
        />
      </n-form-item>
      
      <n-divider />
      
      <n-form-item label="启用代理">
        <n-switch v-model:value="proxyEnabled" />
      </n-form-item>
      
      <n-form-item label="代理地址" v-if="proxyEnabled">
        <n-grid :cols="2" :x-gap="12">
          <n-gi>
            <n-input 
              v-model:value="proxyHost" 
              placeholder="127.0.0.1"
            />
          </n-gi>
          <n-gi>
            <n-input-number 
              v-model:value="proxyPort" 
              :min="1"
              :max="65535"
              placeholder="端口"
              style="width: 100%"
            />
          </n-gi>
        </n-grid>
      </n-form-item>
      
      <n-divider />
      
      <n-form-item>
        <n-space>
          <n-button type="primary" @click="saveSettings">保存设置</n-button>
        </n-space>
      </n-form-item>
    </n-form>
  </div>
</template>

<style scoped>
.settings-view {
  padding: 0;
}
</style>
