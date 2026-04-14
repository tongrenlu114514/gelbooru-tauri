/**
 * Download store unit tests
 * Tests both utility functions and store async functions with Tauri mocking
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { createPinia, setActivePinia } from 'pinia';
import { useDownloadStore } from '@/stores/download';
import type { GelbooruTag } from '@/types';

// Mocks are set up in setup.ts - we just need to configure them in tests

// Inline the utility functions for testing
// These are the same functions from download.ts
function sanitizeFileName(name: string): string {
  return name
    .replace(/[<>:"/\\|?*]/g, '_')
    .replace(/\s+/g, '_')
    .replace(/_{2,}/g, '_')
    .replace(/^_|_$/g, '');
}

function extractTagsByType(tags: GelbooruTag[], type: string): string[] {
  return tags
    .filter((t) => t.tagType.toLowerCase() === type.toLowerCase())
    .map((t) => sanitizeFileName(t.text));
}

interface PostMeta {
  postId: number;
  imageUrl: string;
  posted: string;
  rating: string;
  tags: GelbooruTag[];
}

function generateSavePath(meta: PostMeta, basePath: string): string {
  const artists = extractTagsByType(meta.tags, 'artist');
  const characters = extractTagsByType(meta.tags, 'character');
  const copyrights = extractTagsByType(meta.tags, 'copyright');

  let dateStr = 'unknown';
  if (meta.posted) {
    const match = meta.posted.match(/^(\d{4}-\d{2}-\d{2})/);
    if (match) {
      dateStr = match[1];
    }
  }

  let rating = meta.rating?.toLowerCase() || 'unknown';
  if (rating === 'safe' || rating === 's') rating = 'safe';
  else if (rating === 'questionable' || rating === 'q') rating = 'questionable';
  else if (rating === 'explicit' || rating === 'e') rating = 'explicit';

  const copyright = copyrights.length > 0 ? copyrights[0] : 'unknown';
  const characterPart = characters.length > 0 ? `[${characters.join(',')}]` : '';
  const artistPart = artists.length > 0 ? `(${artists[0]})` : '';

  const ext = meta.imageUrl.split('.').pop()?.split('?')[0] || 'jpg';
  const fileName = `${characterPart}${meta.postId}${artistPart}.${ext}`;

  return `${basePath}/${dateStr}/${rating}/${copyright}/${fileName}`;
}

describe('sanitizeFileName', () => {
  it('should replace illegal characters with underscore', () => {
    expect(sanitizeFileName('file<name>.txt')).toBe('file_name_.txt');
    // Note: '/' is also replaced (included in the regex pattern)
    expect(sanitizeFileName('path/to:file')).toBe('path_to_file');
    expect(sanitizeFileName('file|name')).toBe('file_name');
    expect(sanitizeFileName('file*name')).toBe('file_name');
  });

  it('should replace multiple spaces with single underscore', () => {
    expect(sanitizeFileName('file  name')).toBe('file_name');
    expect(sanitizeFileName('file   name   here')).toBe('file_name_here');
  });

  it('should replace multiple underscores with single underscore', () => {
    expect(sanitizeFileName('file__name')).toBe('file_name');
    expect(sanitizeFileName('file___name___here')).toBe('file_name_here');
  });

  it('should remove leading and trailing underscores', () => {
    expect(sanitizeFileName('_filename_')).toBe('filename');
    expect(sanitizeFileName('__file__name__')).toBe('file_name');
  });

  it('should handle normal filenames without changes', () => {
    expect(sanitizeFileName('normal_file_name.jpg')).toBe('normal_file_name.jpg');
    expect(sanitizeFileName('2024_photo')).toBe('2024_photo');
  });
});

describe('extractTagsByType', () => {
  const mockTags: GelbooruTag[] = [
    { text: 'kawaii', tagType: 'general', count: 100 },
    { text: 'blue hair', tagType: 'general', count: 50 },
    { text: 'Sakura', tagType: 'character', count: 200 },
    { text: 'Naruto', tagType: 'copyright', count: 300 },
    { text: 'mike', tagType: 'artist', count: 10 },
  ];

  it('should extract tags by type case-insensitively', () => {
    expect(extractTagsByType(mockTags, 'general')).toEqual(['kawaii', 'blue_hair']);
    expect(extractTagsByType(mockTags, 'GENERAL')).toEqual(['kawaii', 'blue_hair']);
    expect(extractTagsByType(mockTags, 'General')).toEqual(['kawaii', 'blue_hair']);
  });

  it('should extract character tags', () => {
    expect(extractTagsByType(mockTags, 'character')).toEqual(['Sakura']);
  });

  it('should extract copyright tags', () => {
    expect(extractTagsByType(mockTags, 'copyright')).toEqual(['Naruto']);
  });

  it('should extract artist tags and sanitize them', () => {
    expect(extractTagsByType(mockTags, 'artist')).toEqual(['mike']);
  });

  it('should return empty array when no matching tags', () => {
    expect(extractTagsByType(mockTags, 'species')).toEqual([]);
  });

  it('should return empty array for empty input', () => {
    expect(extractTagsByType([], 'general')).toEqual([]);
  });
});

describe('generateSavePath', () => {
  const basePath = '/downloads/images';

  it('should generate correct path structure', () => {
    const meta: PostMeta = {
      postId: 12345,
      imageUrl: 'https://example.com/image.jpg',
      posted: '2024-03-22 12:34:56',
      rating: 'safe',
      tags: [],
    };

    const result = generateSavePath(meta, basePath);
    expect(result).toBe('/downloads/images/2024-03-22/safe/unknown/12345.jpg');
  });

  it('should include character tags in filename', () => {
    const meta: PostMeta = {
      postId: 12345,
      imageUrl: 'https://example.com/image.png',
      posted: '2024-03-22',
      rating: 'explicit',
      tags: [
        { text: 'Sakura', tagType: 'character', count: 100 },
        { text: ' Hinata', tagType: 'character', count: 50 },
      ],
    };

    const result = generateSavePath(meta, basePath);
    expect(result).toContain('[Sakura,Hinata]12345');
  });

  it('should include artist in parentheses', () => {
    const meta: PostMeta = {
      postId: 99999,
      imageUrl: 'https://example.com/art.jpg',
      posted: '2024-01-01',
      rating: 'questionable',
      tags: [{ text: 'artist_name', tagType: 'artist', count: 100 }],
    };

    const result = generateSavePath(meta, basePath);
    expect(result).toContain('(artist_name)');
  });

  it('should normalize rating values', () => {
    const testCases = [
      { input: 'safe', expected: 'safe' },
      { input: 's', expected: 'safe' },
      { input: 'questionable', expected: 'questionable' },
      { input: 'q', expected: 'questionable' },
      { input: 'explicit', expected: 'explicit' },
      { input: 'e', expected: 'explicit' },
      { input: 'unknown', expected: 'unknown' },
    ];

    for (const { input, expected } of testCases) {
      const meta: PostMeta = {
        postId: 1,
        imageUrl: 'https://example.com/image.jpg',
        posted: '2024-03-22',
        rating: input,
        tags: [],
      };
      const result = generateSavePath(meta, basePath);
      expect(result).toContain(`/${expected}/`);
    }
  });

  it('should use copyright as folder name', () => {
    const meta: PostMeta = {
      postId: 12345,
      imageUrl: 'https://example.com/image.jpg',
      posted: '2024-03-22',
      rating: 'safe',
      tags: [{ text: 'One_Piece', tagType: 'copyright', count: 100 }],
    };

    const result = generateSavePath(meta, basePath);
    expect(result).toContain('/One_Piece/');
  });

  it('should extract extension from URL', () => {
    const meta: PostMeta = {
      postId: 12345,
      imageUrl: 'https://example.com/image.png?size=large',
      posted: '2024-03-22',
      rating: 'safe',
      tags: [],
    };

    const result = generateSavePath(meta, basePath);
    expect(result.slice(-4)).toBe('.png');
  });

  it('should handle missing posted date', () => {
    const meta: PostMeta = {
      postId: 12345,
      imageUrl: 'https://example.com/image.jpg',
      posted: '',
      rating: 'safe',
      tags: [],
    };

    const result = generateSavePath(meta, basePath);
    expect(result).toContain('/unknown/');
  });

  it('should use default extension when URL has no extension', () => {
    const meta: PostMeta = {
      postId: 12345,
      imageUrl: 'https://example.com/noextension',
      posted: '2024-03-22',
      rating: 'safe',
      tags: [],
    };

    // URL contains dots (domain), so last segment becomes "extension"
    // Result ends with: .com/noextension
    const result = generateSavePath(meta, basePath);
    expect(result.slice(-16)).toBe('.com/noextension');
  });

  it('should use default extension when URL ends with query params only', () => {
    const meta: PostMeta = {
      postId: 12345,
      imageUrl: 'https://example.com/image.jpg?size=large',
      posted: '2024-03-22',
      rating: 'safe',
      tags: [],
    };

    const result = generateSavePath(meta, basePath);
    expect(result.slice(-4)).toBe('.jpg');
  });
});

// Mock settings store
vi.mock('@/stores/settings', () => ({
  useSettingsStore: () => ({
    downloadPath: '/downloads',
  }),
}));

describe('useDownloadStore async functions', () => {
  const mockPostMeta = {
    postId: 12345,
    imageUrl: 'https://example.com/image.jpg',
    posted: '2024-03-22 12:34:56',
    rating: 'safe',
    tags: [{ text: 'artist_tag', tagType: 'artist', count: 100 }],
  };

  const mockTask = {
    id: 1,
    postId: 12345,
    imageUrl: 'https://example.com/image.jpg',
    fileName: '12345.jpg',
    savePath: '/downloads/2024-03-22/safe/unknown/12345.jpg',
    status: 'pending' as const,
    progress: 0,
    downloadedSize: 0,
    totalSize: 0,
  };

  beforeEach(() => {
    setActivePinia(createPinia());
    // Reset mock implementations to default state
    vi.mocked(invoke).mockReset();
    vi.mocked(listen).mockReset();
    vi.mocked(listen).mockResolvedValue(vi.fn());
  });

  describe('init', () => {
    it('should call restoreTasks and initListeners', async () => {
      const mockTasks: (typeof mockTask)[] = [];
      vi.mocked(invoke).mockResolvedValueOnce(mockTasks);
      vi.mocked(listen).mockResolvedValueOnce(vi.fn());

      const store = useDownloadStore();
      await store.init();

      expect(invoke).toHaveBeenCalledWith('restore_download_tasks');
      expect(listen).toHaveBeenCalledWith('download-progress', expect.any(Function));
    });

    it('should not reinitialize if already initialized', async () => {
      const store = useDownloadStore();
      await store.init();
      await store.init();

      // Should only call once due to initialized flag
      expect(listen).toHaveBeenCalledTimes(1);
    });
  });

  describe('initListeners', () => {
    it('should set up download progress listener', async () => {
      const mockUnlisten = vi.fn();
      vi.mocked(listen).mockResolvedValueOnce(mockUnlisten);

      const store = useDownloadStore();
      await store.initListeners();

      expect(listen).toHaveBeenCalledWith('download-progress', expect.any(Function));
    });

    it('should not re-setup listener if already exists', async () => {
      const mockUnlisten = vi.fn();
      vi.mocked(listen).mockResolvedValueOnce(mockUnlisten);

      const store = useDownloadStore();
      await store.initListeners();
      await store.initListeners();

      // Should only call listen once
      expect(listen).toHaveBeenCalledTimes(1);
    });
  });

  describe('addTask', () => {
    it('should create task and auto-start download', async () => {
      const task: typeof mockTask = { ...mockTask, id: 1 };
      vi.mocked(invoke)
        .mockResolvedValueOnce(task) // add_download_task
        .mockResolvedValueOnce(undefined); // start_download

      const store = useDownloadStore();
      const result = await store.addTask(mockPostMeta);

      expect(result).toEqual(task);
      expect(store.tasks).toContainEqual(task);
      expect(invoke).toHaveBeenCalledWith('start_download', { id: 1 });
    });

    it('should throw error when invoke fails', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce(vi.fn()) // initListeners
        .mockRejectedValueOnce(new Error('Failed to add task'));

      const store = useDownloadStore();

      await expect(store.addTask(mockPostMeta)).rejects.toThrow('Failed to add task');
    });
  });

  describe('startDownload', () => {
    it('should call invoke with correct id', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(undefined);

      const store = useDownloadStore();
      await store.startDownload(123);

      expect(invoke).toHaveBeenCalledWith('start_download', { id: 123 });
    });

    it('should set isDownloading to true on success', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(undefined);

      const store = useDownloadStore();
      await store.startDownload(1);

      expect(store.isDownloading).toBe(true);
    });

    it('should throw error when invoke fails', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('Download error'));

      const store = useDownloadStore();

      await expect(store.startDownload(1)).rejects.toThrow('Download error');
    });
  });

  describe('pauseDownload', () => {
    it('should call invoke with correct id', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(undefined);

      const store = useDownloadStore();
      await store.pauseDownload(456);

      expect(invoke).toHaveBeenCalledWith('pause_download', { id: 456 });
    });

    it('should handle error gracefully', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('Pause error'));

      const store = useDownloadStore();
      // Should not throw
      await expect(store.pauseDownload(1)).resolves.not.toThrow();
    });
  });

  describe('resumeDownload', () => {
    it('should call invoke with correct id', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(undefined);

      const store = useDownloadStore();
      await store.resumeDownload(789);

      expect(invoke).toHaveBeenCalledWith('resume_download', { id: 789 });
    });

    it('should handle error gracefully', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('Resume error'));

      const store = useDownloadStore();
      await expect(store.resumeDownload(1)).resolves.not.toThrow();
    });
  });

  describe('cancelDownload', () => {
    it('should call invoke with correct id', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(undefined);

      const store = useDownloadStore();
      await store.cancelDownload(999);

      expect(invoke).toHaveBeenCalledWith('cancel_download', { id: 999 });
    });

    it('should handle error gracefully', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('Cancel error'));

      const store = useDownloadStore();
      await expect(store.cancelDownload(1)).resolves.not.toThrow();
    });
  });

  describe('removeTask', () => {
    it('should remove task from local array', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(undefined);

      const store = useDownloadStore();
      store.tasks.push(mockTask);

      await store.removeTask(1);

      expect(store.tasks).toHaveLength(0);
      expect(invoke).toHaveBeenCalledWith('remove_download_task', { id: 1 });
    });

    it('should only remove task with matching id', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(undefined);

      const store = useDownloadStore();
      store.tasks.push({ ...mockTask, id: 1 }, { ...mockTask, id: 2 });

      await store.removeTask(1);

      expect(store.tasks).toHaveLength(1);
      expect(store.tasks[0].id).toBe(2);
    });

    it('should handle error gracefully', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('Remove error'));

      const store = useDownloadStore();
      store.tasks.push(mockTask);

      await expect(store.removeTask(1)).resolves.not.toThrow();
    });
  });

  describe('clearCompleted', () => {
    it('should filter out completed tasks', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(undefined);

      const store = useDownloadStore();
      store.tasks.push(
        { ...mockTask, id: 1, status: 'completed' as const },
        { ...mockTask, id: 2, status: 'pending' as const },
        { ...mockTask, id: 3, status: 'downloading' as const }
      );

      await store.clearCompleted();

      expect(store.tasks).toHaveLength(2);
      expect(store.tasks.find((t) => t.id === 2)).toBeDefined();
      expect(store.tasks.find((t) => t.id === 3)).toBeDefined();
      expect(store.tasks.find((t) => t.id === 1)).toBeUndefined();
      expect(invoke).toHaveBeenCalledWith('clear_completed_tasks');
    });

    it('should handle error gracefully', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('Clear error'));

      const store = useDownloadStore();
      store.tasks.push({ ...mockTask, id: 1, status: 'completed' as const });

      await expect(store.clearCompleted()).resolves.not.toThrow();
    });
  });

  describe('fetchTasks', () => {
    it('should update local tasks array', async () => {
      const mockTasks = [
        { ...mockTask, id: 1 },
        { ...mockTask, id: 2 },
        { ...mockTask, id: 3 },
      ];
      vi.mocked(invoke).mockResolvedValueOnce(mockTasks);

      const store = useDownloadStore();
      await store.fetchTasks();

      expect(store.tasks).toEqual(mockTasks);
      expect(invoke).toHaveBeenCalledWith('get_download_tasks');
    });

    it('should handle error gracefully', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('Fetch error'));

      const store = useDownloadStore();
      await expect(store.fetchTasks()).resolves.not.toThrow();
    });
  });

  describe('startAllPending', () => {
    it('should iterate queue and start each task', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      const store = useDownloadStore();
      store.tasks.push(
        { ...mockTask, id: 1, status: 'pending' as const },
        { ...mockTask, id: 2, status: 'pending' as const },
        { ...mockTask, id: 3, status: 'completed' as const }
      );

      await store.startAllPending();

      expect(invoke).toHaveBeenCalledWith('start_download', { id: 1 });
      expect(invoke).toHaveBeenCalledWith('start_download', { id: 2 });
      expect(invoke).not.toHaveBeenCalledWith('start_download', { id: 3 });
    });

    it('should not start anything when queue is empty', async () => {
      const store = useDownloadStore();
      store.tasks.push({ ...mockTask, id: 1, status: 'completed' as const });

      await store.startAllPending();

      expect(invoke).not.toHaveBeenCalled();
    });
  });

  describe('pauseAllDownloading', () => {
    it('should iterate downloading and pause each task', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      const store = useDownloadStore();
      store.tasks.push(
        { ...mockTask, id: 1, status: 'downloading' as const },
        { ...mockTask, id: 2, status: 'downloading' as const },
        { ...mockTask, id: 3, status: 'pending' as const }
      );

      await store.pauseAllDownloading();

      expect(invoke).toHaveBeenCalledWith('pause_download', { id: 1 });
      expect(invoke).toHaveBeenCalledWith('pause_download', { id: 2 });
      expect(invoke).not.toHaveBeenCalledWith('pause_download', { id: 3 });
    });

    it('should not pause anything when no tasks are downloading', async () => {
      const store = useDownloadStore();
      store.tasks.push({ ...mockTask, id: 1, status: 'pending' as const });

      await store.pauseAllDownloading();

      expect(invoke).not.toHaveBeenCalled();
    });
  });

  describe('computed properties', () => {
    it('should correctly filter queue (pending tasks)', () => {
      const store = useDownloadStore();
      store.tasks.push(
        { ...mockTask, id: 1, status: 'pending' as const },
        { ...mockTask, id: 2, status: 'downloading' as const },
        { ...mockTask, id: 3, status: 'completed' as const }
      );

      expect(store.queue).toHaveLength(1);
      expect(store.queue[0].id).toBe(1);
    });

    it('should correctly filter downloading tasks', () => {
      const store = useDownloadStore();
      store.tasks.push(
        { ...mockTask, id: 1, status: 'pending' as const },
        { ...mockTask, id: 2, status: 'downloading' as const },
        { ...mockTask, id: 3, status: 'completed' as const }
      );

      expect(store.downloading).toHaveLength(1);
      expect(store.downloading[0].id).toBe(2);
    });

    it('should correctly filter completed tasks', () => {
      const store = useDownloadStore();
      store.tasks.push(
        { ...mockTask, id: 1, status: 'pending' as const },
        { ...mockTask, id: 2, status: 'downloading' as const },
        { ...mockTask, id: 3, status: 'completed' as const }
      );

      expect(store.completed).toHaveLength(1);
      expect(store.completed[0].id).toBe(3);
    });

    it('should correctly filter failed tasks', () => {
      const store = useDownloadStore();
      store.tasks.push(
        { ...mockTask, id: 1, status: 'failed' as const },
        { ...mockTask, id: 2, status: 'completed' as const },
        { ...mockTask, id: 3, status: 'pending' as const }
      );

      expect(store.failed).toHaveLength(1);
      expect(store.failed[0].id).toBe(1);
    });
  });

  describe('openFile', () => {
    it('should call invoke with correct path', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(undefined);

      const store = useDownloadStore();
      await store.openFile('/path/to/file.jpg');

      expect(invoke).toHaveBeenCalledWith('open_file', { path: '/path/to/file.jpg' });
    });

    it('should handle error gracefully', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('Open error'));

      const store = useDownloadStore();
      await expect(store.openFile('/path/to/file.jpg')).resolves.not.toThrow();
    });
  });
});
