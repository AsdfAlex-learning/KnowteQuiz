<template>
  <div class="h-full flex flex-col">
    <!-- Filter bar -->
    <div class="px-3 py-2 border-b border-[var(--border-default)] flex items-center gap-2">
      <button
        v-for="f in filters"
        :key="f.value"
        class="px-2.5 py-1 text-xs rounded-md transition-colors"
        :class="
          filter === f.value
            ? 'bg-[var(--accent-purple)]/20 text-[var(--accent-purple)]'
            : 'text-[var(--text-muted)] hover:bg-[var(--bg-elevated)]'
        "
        @click="filter = f.value"
      >
        {{ f.label }}
      </button>
      <span class="ml-auto text-[11px] text-[var(--text-faint)]">
        {{ filteredMistakes.length }} items
      </span>
    </div>

    <!-- Detail view -->
    <div v-if="selectedId" class="flex-1 overflow-y-auto p-3">
      <MistakeDetail
        :mistake="selectedMistake!"
        @back="selectedId = null"
        @open-note="handleOpenNote"
      />
    </div>

    <!-- List view -->
    <div v-else class="flex-1 overflow-y-auto p-2 space-y-1">
      <div
        v-if="loading"
        class="flex items-center justify-center py-8 text-sm text-[var(--text-faint)]"
      >
        Loading...
      </div>
      <div
        v-else-if="filteredMistakes.length === 0"
        class="flex flex-col items-center justify-center py-12 text-[var(--text-faint)]"
      >
        <p class="text-sm">No mistakes recorded</p>
        <p class="text-xs mt-1">Wrong answers will appear here</p>
      </div>
      <MistakeItem
        v-for="m in filteredMistakes"
        :key="m.id"
        :mistake="m"
        :active="selectedId === m.id"
        @select="selectedId = m.id"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { loadMistakes } from '@/services/mistake'
import { readNote } from '@/services/note'
import { useExplorerStore } from '@/stores/explorer'
import MistakeItem from './MistakeItem.vue'
import MistakeDetail from './MistakeDetail.vue'
import type { MistakeEntry, MistakeMode } from '@/types/mistake'

type FilterValue = 'all' | 'basic' | 'advanced'

const filters: { value: FilterValue; label: string }[] = [
  { value: 'all', label: 'All' },
  { value: 'basic', label: 'Basic' },
  { value: 'advanced', label: 'Advanced' },
]

const filter = ref<FilterValue>('all')
const mistakes = ref<MistakeEntry[]>([])
const loading = ref(false)
const selectedId = ref<string | null>(null)
const explorerStore = useExplorerStore()

const filteredMistakes = computed(() => {
  const list = filter.value === 'all'
    ? mistakes.value
    : mistakes.value.filter((m) => m.mode === filter.value)
  return list.sort(
    (a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
  )
})

const selectedMistake = computed(() =>
  mistakes.value.find((m) => m.id === selectedId.value) ?? null
)

function handleOpenNote(path: string) {
  explorerStore.selectPath(path)
  readNote(path)
}

onMounted(async () => {
  loading.value = true
  try {
    mistakes.value = await loadMistakes()
  } finally {
    loading.value = false
  }
})
</script>
