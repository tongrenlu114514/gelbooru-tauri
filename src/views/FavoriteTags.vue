<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import {
  NSpace,
  NButton,
  NInput,
  NTag,
  NEmpty,
  NSpin,
  NModal,
  NSelect,
  NIcon,
  NCollapse,
  NCollapseItem,
  NPopconfirm,
  useMessage
} from 'naive-ui'
import { 
  HeartOutline, 
  AddOutline, 
  TrashOutline, 
  SearchOutline
} from '@vicons/ionicons5'
import { useFavoriteTagsStore } from '@/stores/favoriteTags'
import { useRouter } from 'vue-router'
import type { FavoriteTag } from '@/types'

const router = useRouter()
const message = useMessage()
const favoriteTagsStore = useFavoriteTagsStore()

const showAddModal = ref(false)
const showAddChildModal = ref(false)
const newTagName = ref('')
const newTagType = ref('general')
const selectedParentId = ref<number | null>(null)
const searchQuery = ref('')

const tagTypeOptions = [
  { label: '作品 (Copyright)', value: 'copyright' },
  { label: '角色 (Character)', value: 'character' },
  { label: '艺术家 (Artist)', value: 'artist' },
  { label: '一般 (General)', value: 'general' }
]

const tagTypeColors: Record<string, string> = {
  artist: '#4caf50',
  character: '#e91e63',
  copyright: '#9c27b0',
  general: '#2196f3',
  metadata: '#607d8b'
}

// 过滤后的标签组
const filteredGroups = computed(() => {
  if (!searchQuery.value) {
    return favoriteTagsStore.tags
  }
  
  const query = searchQuery.value.toLowerCase()
  return favoriteTagsStore.tags.filter(group => {
    const parentMatch = group.parent.tag.toLowerCase().includes(query)
    const childMatch = group.children.some(child => 
      child.tag.toLowerCase().includes(query)
    )
    return parentMatch || childMatch
  })
})

// 统计总数
const totalTags = computed(() => {
  let count = 0
  for (const group of favoriteTagsStore.tags) {
    count += 1 // parent
    count += group.children.length
  }
  return count
})

async function openAddModal(parentId?: number) {
  newTagName.value = ''
  newTagType.value = 'general'
  if (parentId) {
    selectedParentId.value = parentId
    showAddChildModal.value = true
  } else {
    selectedParentId.value = null
    showAddModal.value = true
  }
}

async function addTag() {
  if (!newTagName.value.trim()) {
    message.warning('请输入标签名称')
    return
  }
  
  try {
    const tag = newTagName.value.trim().replace(/\s+/g, '_')
    
    if (selectedParentId.value) {
      await favoriteTagsStore.addChildTag(tag, newTagType.value, selectedParentId.value)
      showAddChildModal.value = false
    } else {
      await favoriteTagsStore.addParentTag(tag, newTagType.value)
      showAddModal.value = false
    }
    
    message.success('添加成功')
    newTagName.value = ''
  } catch (error) {
    message.error('添加失败')
  }
}

async function removeTag(tag: FavoriteTag) {
  try {
    await favoriteTagsStore.removeTag(tag.id)
    message.success('删除成功')
  } catch (error) {
    message.error('删除失败')
  }
}

function searchWithTag(tag: string) {
  router.push({ name: 'home', query: { tag } })
}

onMounted(() => {
  favoriteTagsStore.loadTags()
})
</script>

<template>
  <div class="favorite-tags-view">
    <n-space justify="space-between" align="center" style="margin-bottom: 16px;">
      <n-space align="center">
        <n-icon :size="24" color="#e91e63"><HeartOutline /></n-icon>
        <span style="font-size: 18px; font-weight: 500;">收藏标签</span>
        <n-tag size="small" type="info">{{ totalTags }} 个标签</n-tag>
      </n-space>
      <n-space>
        <n-input
          v-model:value="searchQuery"
          placeholder="搜索标签..."
          style="width: 200px"
          clearable
        >
          <template #prefix>
            <n-icon><SearchOutline /></n-icon>
          </template>
        </n-input>
        <n-button type="primary" @click="openAddModal()">
          <template #icon>
            <n-icon><AddOutline /></n-icon>
          </template>
          添加标签
        </n-button>
      </n-space>
    </n-space>
    
    <n-spin :show="favoriteTagsStore.loading">
      <div v-if="filteredGroups.length > 0" class="tags-container">
        <n-collapse>
          <n-collapse-item
            v-for="group in filteredGroups"
            :key="group.parent.id"
            :name="group.parent.id"
          >
            <template #header>
              <div class="group-header">
                <n-tag
                  :color="{ color: tagTypeColors[group.parent.tagType] || tagTypeColors.general, textColor: '#fff' }"
                  size="medium"
                  round
                >
                  {{ group.parent.tag }}
                </n-tag>
                <span class="group-count">{{ group.children.length + 1 }}</span>
              </div>
            </template>
            <template #header-extra>
              <n-space @click.stop>
                <n-button size="tiny" quaternary @click="openAddModal(group.parent.id)">
                  <template #icon>
                    <n-icon><AddOutline /></n-icon>
                  </template>
                </n-button>
                <n-popconfirm @positive-click="removeTag(group.parent)">
                  <template #trigger>
                    <n-button size="tiny" quaternary type="error">
                      <template #icon>
                        <n-icon><TrashOutline /></n-icon>
                      </template>
                    </n-button>
                  </template>
                  确定删除该标签组吗？子标签也会被删除。
                </n-popconfirm>
              </n-space>
            </template>
            
            <div class="tag-list">
              <!-- Parent tag -->
              <div class="tag-item parent-tag">
                <n-tag
                  :color="{ color: tagTypeColors[group.parent.tagType] || tagTypeColors.general, textColor: '#fff' }"
                  size="large"
                  round
                  clickable
                  @click="searchWithTag(group.parent.tag)"
                >
                  {{ group.parent.tag }}
                </n-tag>
                <n-button 
                  size="tiny" 
                  quaternary 
                  type="primary"
                  @click="searchWithTag(group.parent.tag)"
                >
                  搜索
                </n-button>
              </div>
              
              <!-- Child tags -->
              <div 
                v-for="child in group.children" 
                :key="child.id" 
                class="tag-item child-tag"
              >
                <n-tag
                  :color="{ color: tagTypeColors[child.tagType] || tagTypeColors.general, textColor: '#fff' }"
                  size="medium"
                  round
                  clickable
                  @click="searchWithTag(child.tag)"
                >
                  {{ child.tag }}
                </n-tag>
                <n-space>
                  <n-button 
                    size="tiny" 
                    quaternary 
                    type="primary"
                    @click="searchWithTag(child.tag)"
                  >
                    搜索
                  </n-button>
                  <n-popconfirm @positive-click="removeTag(child)">
                    <template #trigger>
                      <n-button size="tiny" quaternary type="error">
                        <template #icon>
                          <n-icon><TrashOutline /></n-icon>
                        </template>
                      </n-button>
                    </template>
                    确定删除该标签吗？
                  </n-popconfirm>
                </n-space>
              </div>
              
              <!-- Empty children hint -->
              <div v-if="group.children.length === 0" class="empty-children">
                <span>暂无子标签，点击上方 + 添加</span>
              </div>
            </div>
          </n-collapse-item>
        </n-collapse>
      </div>
      
      <n-empty v-else description="暂无收藏标签">
        <template #extra>
          <n-button type="primary" @click="openAddModal()">
            添加第一个标签
          </n-button>
        </template>
      </n-empty>
    </n-spin>
    
    <!-- Add Parent Tag Modal -->
    <n-modal
      v-model:show="showAddModal"
      preset="card"
      title="添加收藏标签"
      style="width: 400px"
    >
      <n-space vertical>
        <n-input
          v-model:value="newTagName"
          placeholder="输入标签名称"
          @keyup.enter="addTag"
        />
        <n-select
          v-model:value="newTagType"
          :options="tagTypeOptions"
          placeholder="选择标签类型"
        />
      </n-space>
      <template #footer>
        <n-space justify="end">
          <n-button @click="showAddModal = false">取消</n-button>
          <n-button type="primary" @click="addTag">添加</n-button>
        </n-space>
      </template>
    </n-modal>
    
    <!-- Add Child Tag Modal -->
    <n-modal
      v-model:show="showAddChildModal"
      preset="card"
      title="添加子标签"
      style="width: 400px"
    >
      <n-space vertical>
        <n-input
          v-model:value="newTagName"
          placeholder="输入子标签名称"
          @keyup.enter="addTag"
        />
        <n-select
          v-model:value="newTagType"
          :options="tagTypeOptions"
          placeholder="选择标签类型"
        />
      </n-space>
      <template #footer>
        <n-space justify="end">
          <n-button @click="showAddChildModal = false">取消</n-button>
          <n-button type="primary" @click="addTag">添加</n-button>
        </n-space>
      </template>
    </n-modal>
  </div>
</template>

<style scoped>
.favorite-tags-view {
  padding: 0;
}

.tags-container {
  background: rgba(255, 255, 255, 0.02);
  border-radius: 8px;
  padding: 4px;
}

.group-header {
  display: flex;
  align-items: center;
  gap: 12px;
}

.group-count {
  font-size: 12px;
  color: #999;
  background: rgba(255, 255, 255, 0.1);
  padding: 2px 8px;
  border-radius: 10px;
}

.tag-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 8px 0;
}

.tag-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 8px;
  border-radius: 6px;
  transition: background 0.2s;
}

.tag-item:hover {
  background: rgba(255, 255, 255, 0.05);
}

.parent-tag {
  background: rgba(255, 255, 255, 0.03);
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  margin-bottom: 8px;
  padding-bottom: 12px;
}

.child-tag {
  padding-left: 24px;
}

.empty-children {
  padding: 16px;
  text-align: center;
  color: #999;
  font-size: 13px;
}
</style>
