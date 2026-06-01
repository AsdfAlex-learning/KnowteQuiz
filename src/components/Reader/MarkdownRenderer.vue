<template>
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
</template>

<script setup lang="ts">
import { computed } from 'vue'
import MarkdownIt from 'markdown-it'
import mk from '@traptitech/markdown-it-katex'
import hljs from 'highlight.js'
import 'katex/dist/katex.min.css'
import { useReaderStore } from '@/stores/reader'
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

const renderedHtml = computed(() => {
  if (!readerStore.currentNote?.content) return ''
  return md.render(readerStore.currentNote.content)
})
</script>