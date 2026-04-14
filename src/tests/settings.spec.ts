/**
 * Settings store unit tests
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { setActivePinia, createPinia } from 'pinia';
import { useSettingsStore } from '@/stores/settings';

// Type augmentation for testing
interface MockAppSettings {
  theme: 'light' | 'dark';
  sidebarCollapsed: boolean;
  downloadPath: string;
  concurrentDownloads: number;
  proxyEnabled: boolean;
  proxyHost: string;
  proxyPort: number;
}

vi.mock('@tauri-apps/api/core');

describe('useSettingsStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('should initialize with default values', () => {
    const store = useSettingsStore();

    expect(store.theme).toBe('dark');
    expect(store.sidebarCollapsed).toBe(false);
    expect(store.downloadPath).toBe('');
    expect(store.concurrentDownloads).toBe(3);
    expect(store.proxyEnabled).toBe(true);
    expect(store.proxyHost).toBe('127.0.0.1');
    expect(store.proxyPort).toBe(7897);
  });

  describe('loadSettings', () => {
    it('should load settings from backend', async () => {
      const mockSettings: MockAppSettings = {
        theme: 'light',
        sidebarCollapsed: true,
        downloadPath: '/custom/path',
        concurrentDownloads: 5,
        proxyEnabled: false,
        proxyHost: '192.168.1.1',
        proxyPort: 8080,
      };

      vi.mocked(invoke).mockResolvedValueOnce(mockSettings);

      const store = useSettingsStore();
      await store.loadSettings();

      expect(store.theme).toBe('light');
      expect(store.sidebarCollapsed).toBe(true);
      expect(store.downloadPath).toBe('/custom/path');
      expect(store.concurrentDownloads).toBe(5);
      expect(store.proxyEnabled).toBe(false);
      expect(store.proxyHost).toBe('192.168.1.1');
      expect(store.proxyPort).toBe(8080);
    });

    it('should handle load error gracefully', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('Failed to load'));

      const store = useSettingsStore();
      await store.loadSettings();

      // Should keep default values on error
      expect(store.theme).toBe('dark');
      expect(store.downloadPath).toBe('');
    });
  });

  describe('saveSettings', () => {
    it('should save settings to backend', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(undefined);

      const store = useSettingsStore();
      await store.saveSettings();

      expect(invoke).toHaveBeenCalledWith('save_settings', {
        settings: expect.objectContaining({
          theme: 'dark',
          sidebarCollapsed: false,
          concurrentDownloads: 3,
          proxyEnabled: true,
          proxyHost: '127.0.0.1',
          proxyPort: 7897,
        }),
      });
    });
  });

  describe('toggleTheme', () => {
    it('should toggle theme from dark to light', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(undefined);

      const store = useSettingsStore();
      expect(store.theme).toBe('dark');

      store.toggleTheme();
      expect(store.theme).toBe('light');

      // Advance timers for debounced save
      await vi.runAllTimersAsync();
    });

    it('should toggle theme from light to dark', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(undefined);

      const store = useSettingsStore();
      store.theme = 'light';

      store.toggleTheme();
      expect(store.theme).toBe('dark');
    });
  });

  describe('toggleSidebar', () => {
    it('should toggle sidebar collapsed state', () => {
      const store = useSettingsStore();
      expect(store.sidebarCollapsed).toBe(false);

      store.toggleSidebar();
      expect(store.sidebarCollapsed).toBe(true);

      store.toggleSidebar();
      expect(store.sidebarCollapsed).toBe(false);
    });
  });

  describe('updateDownloadPath', () => {
    it('should update download path', () => {
      const store = useSettingsStore();

      store.updateDownloadPath('/new/download/path');
      expect(store.downloadPath).toBe('/new/download/path');
    });
  });

  describe('updateConcurrentDownloads', () => {
    it('should update concurrent downloads count', () => {
      const store = useSettingsStore();

      store.updateConcurrentDownloads(10);
      expect(store.concurrentDownloads).toBe(10);
    });
  });

  describe('updateProxy', () => {
    it('should update proxy enabled state', () => {
      const store = useSettingsStore();

      store.updateProxy(false);
      expect(store.proxyEnabled).toBe(false);
    });

    it('should update proxy host when provided', () => {
      const store = useSettingsStore();

      store.updateProxy(true, '10.0.0.1');
      expect(store.proxyHost).toBe('10.0.0.1');
    });

    it('should update proxy port when provided', () => {
      const store = useSettingsStore();

      store.updateProxy(true, undefined, 3128);
      expect(store.proxyPort).toBe(3128);
    });

    it('should update all proxy settings at once', () => {
      const store = useSettingsStore();

      store.updateProxy(true, '10.0.0.1', 8080);
      expect(store.proxyEnabled).toBe(true);
      expect(store.proxyHost).toBe('10.0.0.1');
      expect(store.proxyPort).toBe(8080);
    });
  });

  describe('getProxyUrl', () => {
    it('should return proxy URL when enabled', () => {
      const store = useSettingsStore();
      store.proxyEnabled = true;
      store.proxyHost = '127.0.0.1';
      store.proxyPort = 7897;

      expect(store.getProxyUrl()).toBe('http://127.0.0.1:7897');
    });

    it('should return empty string when disabled', () => {
      const store = useSettingsStore();
      store.proxyEnabled = false;

      expect(store.getProxyUrl()).toBe('');
    });
  });
});
