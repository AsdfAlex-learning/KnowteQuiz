<template>
  <div class="h-full flex flex-col">
    <!-- Filter bar -->
    <div class="px-3 py-2 border-b border-[var(--border-default)] flex items-center gap-2">
      <button
        v-for="f in filters"
        :key="f.value"
        class="px-2.5 py-1 text-xs rounded-md transition-colors"
        :class="
          activeFilter === f.value
            ? 'bg-[var(--accent-purple)]/20 text-[var(--accent-purple)]'
            : 'text-[var(--text-muted)] hover:bg-[var(--bg-elevated)]'
        "
        @click="handleFilter(f.value)"
      >
        {{ f.label }}
      </button>
      <span class="ml-auto text-[11px] text-[var(--text-faint)]">
        {{ mistakeStore.items.length }} items
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
        v-if="mistakeStore.loading && mistakeStore.items.length === 0"
        class="flex items-center justify-center py-8 text-sm text-[var(--text-faint)]"
      >
        Loading...
      </div>
      <div
        v-else-if="mistakeStore.listError"
        class="flex flex-col items-center justify-center py-12 text-[var(--color-error)]"
      >
        <p class="text-sm">Failed to load mistakes</p>
        <p class="text-xs mt-1">{{ mistakeStore.listError }}</p>
      </div>
      <div
        v-else-if="mistakeStore.items.length === 0"
        class="flex flex-col items-center justify-center py-12 text-[var(--text-faint)]"
      >
        <p class="text-sm">No mistakes recorded</p>
        <p class="text-xs mt-1">Wrong answers will appear here</p>
      </div>
      <MistakeItem
        v-for="m in mistakeStore.items"
        :key="m.id"
        :mistake="m"
        :active="selectedId === m.id"
        @select="selectedId = m.id"
      />
      <button
        v-if="mistakeStore.hasMore"
        class="w-full py-2 rounded-md text-xs font-medium text-[var(--accent-purple)] hover:bg-[var(--bg-elevated)] transition-colors disabled:cursor-wait disabled:text-[var(--text-muted)]"
        :disabled="mistakeStore.loading"
        @click="mistakeStore.loadNextPage()"
      >
        {{ mistakeStore.loading ? 'Loading...' : 'Load more' }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useMistakeStore } from '@/stores/mistakes'
import { useNavigationStore } from '@/stores/navigation'
import MistakeItem from './MistakeItem.vue'
import MistakeDetail from './MistakeDetail.vue'
import type { MistakeMode } from '@/types/mistake'

type FilterValue = 'all' | 'basic' | 'advanced'

const filters: { value: FilterValue; label: string }[] = [
  { value: 'all', label: 'All' },
  { value: 'basic', label: 'Basic' },
  { value: 'advanced', label: 'Advanced' },
]

const selectedId = ref<string | null>(null)
const mistakeStore = useMistakeStore()
const navigationStore = useNavigationStore()

const activeFilter = computed<FilterValue>(() => {
  return mistakeStore.modeFilter ?? 'all'
})

const selectedMistake = computed(() =>
  mistakeStore.items.find((m) => m.id === selectedId.value) ?? null
)

async function handleFilter(value: FilterValue) {
  selectedId.value = null
  await mistakeStore.setModeFilter(value === 'all' ? undefined : value as MistakeMode)
}

function handleOpenNote(path: string) {
  navigationStore.openNote(path)
}

onMounted(async () => {
  await mistakeStore.loadPage()
})
</script>
