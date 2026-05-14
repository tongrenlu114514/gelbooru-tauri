<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { NAutoComplete, NTag } from 'naive-ui';
import { invoke } from '@tauri-apps/api/core';
import { useSearchHistoryStore } from '@/stores/searchHistory';
import type { GelbooruTag } from '@/types';

interface Props {
  modelValue: string[];
}

interface Emits {
  (e: 'update:modelValue', tags: string[]): void;
  (e: 'search', tags: string[]): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const searchHistoryStore = useSearchHistoryStore();

const inputValue = ref('');
const suggestions = ref<GelbooruTag[]>([]);
const loading = ref(false);

function formatCount(count: number): string {
  if (count >= 1000000) return (count / 1000000).toFixed(1) + 'M';
  if (count >= 1000) return (count / 1000).toFixed(1) + 'K';
  return count.toString();
}

const suggestionOptions = computed(() =>
  suggestions.value.map((tag) => ({
    label: tag.text + ' ' + formatCount(tag.count),
    value: tag.text,
  }))
);

async function fetchTagSuggestions(query: string): Promise<GelbooruTag[]> {
  try {
    const result = await invoke<GelbooruTag[]>('search_tags', { query, limit: 8 });
    return result;
  } catch {
    return [];
  }
}

function getHistorySuggestions(): GelbooruTag[] {
  const history = searchHistoryStore.getTopTags(4);
  return history.map((h) => ({
    text: h.tag,
    tagType: 'general',
    count: h.count,
  }));
}

async function loadSuggestions() {
  const query = inputValue.value.trim();
  if (query.length < 2) {
    suggestions.value = [];
    return;
  }

  loading.value = true;
  try {
    // Fetch from API
    const apiTags = await fetchTagSuggestions(query);

    // Merge with history suggestions
    const historyTags = getHistorySuggestions();

    // Combine, dedupe against modelValue, limit to 8
    const selected = new Set(props.modelValue);
    const merged = [...apiTags];
    for (const hist of historyTags) {
      if (!merged.some((t) => t.text === hist.text) && !selected.has(hist.text)) {
        merged.push(hist);
      }
    }

    // Filter out already selected tags
    suggestions.value = merged.filter((t) => !selected.has(t.text)).slice(0, 8);
  } catch {
    // Fall back to history on error
    const historyTags = getHistorySuggestions();
    suggestions.value = historyTags.filter((t) => !props.modelValue.includes(t.text));
  } finally {
    loading.value = false;
  }
}

watch(inputValue, (val) => {
  if (val.length >= 2) {
    loadSuggestions();
  } else {
    suggestions.value = [];
  }
});

function addTag(tag: string) {
  if (!tag || props.modelValue.includes(tag)) return;
  emit('update:modelValue', [...props.modelValue, tag]);
}

function removeTag(tag: string) {
  emit('update:modelValue', props.modelValue.filter((t) => t !== tag));
}

function handleSelect(value: string) {
  addTag(value);
  inputValue.value = '';
  suggestions.value = [];
}

function handleEnter() {
  const query = inputValue.value.trim();
  if (query) {
    addTag(query);
    inputValue.value = '';
    suggestions.value = [];
  }
  // Trigger search with current tags
  emit('search', props.modelValue);
  // Record all current tags to search history
  for (const tag of props.modelValue) {
    searchHistoryStore.recordSearch(tag);
  }
}

function renderSuggestion(tag: { label: string; value: string }) {
  // Render as plain string — NAutoComplete uses this as-is
  return tag.label;
}
</script>

<template>
  <div class="tag-autocomplete">
    <n-auto-complete
      v-model:value="inputValue"
      :options="suggestionOptions"
      :render-label="renderSuggestion"
      placeholder="输入标签搜索..."
      clearable
      @select="handleSelect"
      @keyup.enter="handleEnter"
    />
    <div v-if="modelValue.length > 0" class="selected-tags">
      <n-tag
        v-for="tag in modelValue"
        :key="tag"
        closable
        round
        @close="removeTag(tag)"
      >
        {{ tag }}
      </n-tag>
    </div>
  </div>
</template>

<style scoped>
.tag-autocomplete {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 400px;
}

.selected-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.selected-tags :deep(.n-tag) {
  background: rgba(99, 226, 183, 0.15);
  border: 1px solid rgba(99, 226, 183, 0.3);
  color: #63e2b7;
}
</style>