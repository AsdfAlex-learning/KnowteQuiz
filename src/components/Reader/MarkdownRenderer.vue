<template>
  <div class="flex-1 flex flex-col">
    <!-- Search bar -->
    <div v-if="showSearch" class="flex items-center gap-2 px-4 py-2 border-b border-[var(--border-default)] bg-[var(--bg-elevated)]">
      <input
        ref="searchInput"
        v-model="searchQuery"
        type="text"
        placeholder="Find in note..."
        class="flex-1 px-2.5 py-1 text-xs bg-[var(--bg-base)] text-[var(--text-primary)] border border-[var(--border-default)] rounded focus:border-[var(--border-focus)] focus:outline-none placeholder:text-[var(--text-faint)]"
        @input="handleSearchInput"
        @keydown.enter.prevent="findNext"
        @keydown.escape="closeSearch"
      />
      <span class="text-[11px] text-[var(--text-muted)] min-w-[40px] text-center">{{ matchInfo }}</span>
      <button
        class="px-2 py-0.5 text-[11px] rounded text-[var(--text-muted)] hover:bg-[var(--bg-active)] transition-colors"
        title="Previous match"
        @click="findPrev"
      >
        ▲
      </button>
      <button
        class="px-2 py-0.5 text-[11px] rounded text-[var(--text-muted)] hover:bg-[var(--bg-active)] transition-colors"
        title="Next match"
        @click="findNext"
      >
        ▼
      </button>
      <button
        class="px-2 py-0.5 text-[11px] rounded text-[var(--text-muted)] hover:bg-[var(--bg-active)] transition-colors"
        title="Close search"
        @click="closeSearch"
      >
        ✕
      </button>
    </div>

    <div class="flex-1 overflow-y-auto p-6">
      <!-- Loading state -->
      <LoadingSpinner v-if="readerStore.isLoading" label="Loading note..." overlay />

      <!-- Error state -->
      <div v-else-if="readerStore.error" class="flex flex-col items-center justify-center h-full text-center">
        <svg class="w-12 h-12 text-[var(--color-error)] mb-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <circle cx="12" cy="12" r="10" />
          <path d="M12 8v4m0 4h.01" />
        </svg>
        <p class="text-[var(--text-sm)] text-[var(--color-error)]">{{ readerStore.error }}</p>
      </div>

      <!-- Rendered markdown -->
      <article v-else-if="readerStore.currentNote" class="markdown-body max-w-none" v-html="renderedHtml" />

      <!-- Empty state -->
      <EmptyState v-else />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'
import MarkdownIt from 'markdown-it'
import mk from '@traptitech/markdown-it-katex'
import hljs from 'highlight.js'
import 'katex/dist/katex.min.css'
import { useReaderStore } from '@/stores/reader'
import { convertFileSrc, isTauri } from '@/services/tauri'
import { renderMarkdownWithFallback } from '@/utils/markdown'
import { configureMarkdownAssetRenderer, markdownWebAssetUrl } from '@/utils/markdownAssets'
import EmptyState from './EmptyState.vue'
import LoadingSpinner from '@/components/common/LoadingSpinner.vue'

const readerStore = useReaderStore()

const md = new MarkdownIt({
  html: true,
  linkify: true,
  typographer: true,
  highlight(str: string, lang: string): string {
    if (lang && hljs.getLanguage(lang)) {
      try {
        return `<pre class="hljs"><code>${hljs.highlight(str, { language: lang }).value}</code></pre>`
      } catch {
        // fall through
      }
    }
    return `<pre class="hljs"><code>${md.utils.escapeHtml(str)}</code></pre>`
  },
})

md.use(mk)
configureMarkdownAssetRenderer(
  md,
  () => readerStore.currentNote?.path,
  (path) => isTauri() ? convertFileSrc(path) : markdownWebAssetUrl(path),
)

const renderedHtml = computed(() => {
  if (!readerStore.currentNote?.content) return ''
  return renderMarkdownWithFallback(readerStore.currentNote.content, (source) => md.render(source))
})

// In-note search
const showSearch = ref(false)
const searchQuery = ref('')
const searchInput = ref<HTMLInputElement | null>(null)
const matchInfo = ref('')
let searchTimer: ReturnType<typeof setTimeout> | null = null

function handleSearchInput() {
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(() => {
    doSearch()
  }, 150)
}

function doSearch() {
  if (!searchQuery.value.trim()) {
    clearFind()
    return
  }
  // window.find() highlights and scrolls to the first match
  const found = window.find(searchQuery.value, false, false, true, false, true, false)
  matchInfo.value = found ? '?' : '0/0'
}

function findNext() {
  if (!searchQuery.value.trim()) return
  const found = window.find(searchQuery.value, false, false, true, false, false, true)
  matchInfo.value = found ? '?' : '0/0'
}

function findPrev() {
  if (!searchQuery.value.trim()) return
  const found = window.find(searchQuery.value, false, true, true, false, false, true)
  matchInfo.value = found ? '?' : '0/0'
}

function clearFind() {
  window.getSelection()?.removeAllRanges()
  matchInfo.value = ''
}

function closeSearch() {
  showSearch.value = false
  searchQuery.value = ''
  clearFind()
}

async function openSearch() {
  showSearch.value = true
  await nextTick()
  searchInput.value?.focus()
  if (searchQuery.value) {
    doSearch()
  }
}

function handleKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key === 'f') {
    e.preventDefault()
    openSearch()
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})
</script>
