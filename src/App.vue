<script setup lang="ts">
import { NConfigProvider, NLayout, NLayoutHeader, NLayoutContent, NLayoutSider, NNotificationProvider, NMessageProvider, darkTheme, type GlobalThemeOverrides } from 'naive-ui'
import { computed } from 'vue'
import { useSettingsStore } from '@/stores/settings'
import AppSidebar from '@/components/AppSidebar.vue'
import DownloadNotifier from '@/components/DownloadNotifier.vue'

const settingsStore = useSettingsStore()

const themeOverrides: GlobalThemeOverrides = {
  common: {
    primaryColor: '#63e2b7',
    primaryColorHover: '#7ce2c8',
    primaryColorPressed: '#4bc9a3'
  }
}

const theme = computed(() => settingsStore.theme === 'dark' ? darkTheme : null)
</script>

<template>
  <n-config-provider :theme="theme" :theme-overrides="themeOverrides">
    <n-notification-provider>
      <n-message-provider>
        <download-notifier />
        <n-layout has-sider style="height: 100vh">
          <n-layout-sider
            bordered
            :collapsed="settingsStore.sidebarCollapsed"
            collapse-mode="width"
            :collapsed-width="64"
            :width="200"
            :native-scrollbar="false"
          >
            <app-sidebar />
          </n-layout-sider>
          <n-layout>
            <n-layout-header bordered style="height: 48px; padding: 0 16px; display: flex; align-items: center;">
              <span style="font-size: 18px; font-weight: 500;">Gelbooru Downloader</span>
            </n-layout-header>
            <n-layout-content content-style="padding: 16px;" :native-scrollbar="true">
              <router-view />
            </n-layout-content>
          </n-layout>
        </n-layout>
      </n-message-provider>
    </n-notification-provider>
  </n-config-provider>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}
</style>
