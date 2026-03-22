<script setup lang="ts">
import { NMenu, NButton, NSpace } from 'naive-ui'
import { h } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useSettingsStore } from '@/stores/settings'
import {
  SearchOutline,
  DownloadOutline,
  ImageOutline,
  SettingsOutline,
  MenuOutline
} from '@vicons/ionicons5'
import type { MenuOption } from 'naive-ui'

const router = useRouter()
const route = useRoute()
const settingsStore = useSettingsStore()

const renderIcon = (icon: any) => () => h(icon)

const menuOptions: MenuOption[] = [
  {
    label: '搜索',
    key: 'home',
    icon: renderIcon(SearchOutline)
  },
  {
    label: '下载管理',
    key: 'downloads',
    icon: renderIcon(DownloadOutline)
  },
  {
    label: '本地图库',
    key: 'gallery',
    icon: renderIcon(ImageOutline)
  },
  {
    label: '设置',
    key: 'settings',
    icon: renderIcon(SettingsOutline)
  }
]

function handleMenuSelect(key: string) {
  router.push({ name: key })
}
</script>

<template>
  <div style="height: 100%; display: flex; flex-direction: column;">
    <n-space justify="center" style="padding: 12px;">
      <n-button quaternary circle @click="settingsStore.toggleSidebar">
        <template #icon>
          <MenuOutline />
        </template>
      </n-button>
    </n-space>
    <n-menu
      :value="route.name as string"
      :options="menuOptions"
      :collapsed="settingsStore.sidebarCollapsed"
      :collapsed-width="64"
      :collapsed-icon-size="22"
      @update:value="handleMenuSelect"
    />
  </div>
</template>
