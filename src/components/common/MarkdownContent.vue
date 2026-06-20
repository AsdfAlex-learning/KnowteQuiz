<template>
  <div class="markdown-body" v-html="renderedHtml" />
</template>

<script setup lang="ts">
import { computed } from 'vue'
import MarkdownIt from 'markdown-it'
import mk from '@traptitech/markdown-it-katex'
import { renderMarkdownWithFallback } from '@/utils/markdown'

const md = new MarkdownIt({
  html: true,
  linkify: true,
  typographer: true,
})

md.use(mk)

const props = defineProps<{
  source: string
}>()

const renderedHtml = computed(() => renderMarkdownWithFallback(props.source ?? '', (source) => md.render(source)))
</script>
