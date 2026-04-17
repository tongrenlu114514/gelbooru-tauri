import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { appDataDir } from '@tauri-apps/api/path';

export interface AppSettings {
  theme: 'light' | 'dark';
  sidebarCollapsed: boolean;
  downloadPath: string;
  concurrentDownloads: number;
  proxyEnabled: boolean;
  proxyHost: string;
  proxyPort: number;
}

let saveTimeout: number | null = null;

export const useSettingsStore = defineStore('settings', () => {
  const theme = ref<'light' | 'dark'>('dark');
  const sidebarCollapsed = ref(false);
  const downloadPath = ref('');
  const concurrentDownloads = ref(3);
  const proxyEnabled = ref(true);
  const proxyHost = ref('127.0.0.1');
  const proxyPort = ref(7897);

  async function loadSettings() {
    try {
      const settings = await invoke<AppSettings>('get_settings');
      theme.value = settings.theme as 'light' | 'dark';
      sidebarCollapsed.value = settings.sidebarCollapsed;
      downloadPath.value = settings.downloadPath;
      concurrentDownloads.value = settings.concurrentDownloads;
      proxyEnabled.value = settings.proxyEnabled;
      proxyHost.value = settings.proxyHost;
      proxyPort.value = settings.proxyPort;

      // Set default download path if empty
      if (!downloadPath.value) {
        const defaultPath = await appDataDir();
        downloadPath.value = `${defaultPath}downloads`;
        saveSettingsDebounced();
      }

      // Apply proxy settings
      await applyProxy();
    } catch (error) {
      console.error('Failed to load settings:', error);
    }
  }

  async function applyProxy() {
    try {
      if (proxyEnabled.value) {
        const proxyUrl = `http://${proxyHost.value}:${proxyPort.value}`;
        await invoke('set_proxy', { proxyUrl });
      } else {
        await invoke('set_proxy', { proxyUrl: null });
      }
    } catch (error) {
      console.error('Failed to apply proxy settings:', error);
    }
  }

  function saveSettingsDebounced() {
    if (saveTimeout !== null) {
      clearTimeout(saveTimeout);
    }
    saveTimeout = window.setTimeout(() => {
      saveSettings();
      saveTimeout = null;
    }, 500);
  }

  async function saveSettings() {
    try {
      await invoke('save_settings', {
        settings: {
          theme: theme.value,
          sidebarCollapsed: sidebarCollapsed.value,
          downloadPath: downloadPath.value,
          concurrentDownloads: concurrentDownloads.value,
          proxyEnabled: proxyEnabled.value,
          proxyHost: proxyHost.value,
          proxyPort: proxyPort.value,
        },
      });
    } catch (error) {
      console.error('Failed to save settings:', error);
    }
  }

  // Immediate save without debounce — use when user clicks "保存设置" and
  // needs the backend DB updated before navigating away.
  async function forceSave(): Promise<void> {
    if (saveTimeout !== null) {
      clearTimeout(saveTimeout);
      saveTimeout = null;
    }
    await saveSettings();
  }

  function toggleTheme() {
    theme.value = theme.value === 'dark' ? 'light' : 'dark';
    saveSettingsDebounced();
  }

  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value;
    saveSettingsDebounced();
  }

  function updateDownloadPath(path: string) {
    downloadPath.value = path;
    saveSettingsDebounced();
  }

  function updateConcurrentDownloads(count: number) {
    concurrentDownloads.value = count;
    saveSettingsDebounced();
  }

  function updateProxy(enabled: boolean, host?: string, port?: number) {
    proxyEnabled.value = enabled;
    if (host !== undefined) {
      proxyHost.value = host;
    }
    if (port !== undefined) {
      proxyPort.value = port;
    }
    saveSettingsDebounced();
    // Apply proxy immediately without waiting for debounce
    applyProxy();
  }

  function getProxyUrl(): string {
    if (!proxyEnabled.value) return '';
    return `http://${proxyHost.value}:${proxyPort.value}`;
  }

  return {
    theme,
    sidebarCollapsed,
    downloadPath,
    concurrentDownloads,
    proxyEnabled,
    proxyHost,
    proxyPort,
    loadSettings,
    saveSettings,
    applyProxy,
    toggleTheme,
    toggleSidebar,
    updateDownloadPath,
    updateConcurrentDownloads,
    updateProxy,
    getProxyUrl,
    forceSave,
  };
});
