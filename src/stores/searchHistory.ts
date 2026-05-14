import { defineStore } from 'pinia';
import { ref } from 'vue';

export interface TagFrequency {
  tag: string;
  count: number;
  lastSearched: number; // Unix timestamp ms
}

const STORAGE_KEY = 'gelbooru:search-history';
const MAX_ENTRIES = 100;

function loadFromStorage(): TagFrequency[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    return JSON.parse(raw) as TagFrequency[];
  } catch {
    return [];
  }
}

function saveToStorage(frequencies: TagFrequency[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(frequencies));
  } catch {
    // Silent fallback for localStorage errors
  }
}

export const useSearchHistoryStore = defineStore('searchHistory', () => {
  const frequencies = ref<TagFrequency[]>(loadFromStorage());

  // Persist to localStorage
  function save(): void {
    saveToStorage(frequencies.value);
  }

  // recordSearch(tag): increment count, update timestamp, save
  // If tag not in list, add with count=1. Sort by count desc.
  function recordSearch(tag: string): void {
    if (!tag) return;

    const existing = frequencies.value.find((f) => f.tag === tag);
    if (existing) {
      frequencies.value = frequencies.value.map((f) =>
        f.tag === tag
          ? { ...f, count: f.count + 1, lastSearched: Date.now() }
          : f
      );
    } else {
      frequencies.value = [
        ...frequencies.value,
        { tag, count: 1, lastSearched: Date.now() },
      ];
    }

    // Trim to MAX_ENTRIES, keeping highest frequency
    if (frequencies.value.length > MAX_ENTRIES) {
      const sorted = [...frequencies.value].sort((a, b) => b.count - a.count);
      frequencies.value = sorted.slice(0, MAX_ENTRIES);
    }

    save();
  }

  // getTopTags(limit): return top N sorted by frequency desc
  function getTopTags(limit: number): TagFrequency[] {
    return [...frequencies.value]
      .sort((a, b) => b.count - a.count)
      .slice(0, limit);
  }

  // clearHistory(): reset frequencies to [], save
  function clearHistory(): void {
    frequencies.value = [];
    save();
  }

  return { frequencies, recordSearch, getTopTags, clearHistory };
});