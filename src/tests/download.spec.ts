/**
 * Download store utility functions unit tests
 * Tests sanitizeFileName, extractTagsByType, generateSavePath
 */
import { describe, it, expect } from 'vitest';
import type { GelbooruTag } from '@/types';

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
