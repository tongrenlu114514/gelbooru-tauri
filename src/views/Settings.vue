<script setup lang="ts">
import {
  NForm,
  NFormItem,
  NInput,
  NInputNumber,
  NButton,
  NSpace,
  NSwitch,
  NDivider
} from 'naive-ui'
import { useSettingsStore } from '@/stores/settings'
import { storeToRefs } from 'pinia'

const settingsStore = useSettingsStore()
const { theme, downloadPath, concurrentDownloads } = storeToRefs(settingsStore)

function saveSettings() {
  // Save settings to localStorage or Tauri backend
  localStorage.setItem('settings', JSON.stringify({
    theme: theme.value,
    downloadPath: downloadPath.value,
    concurrentDownloads: concurrentDownloads.value
  }))
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
