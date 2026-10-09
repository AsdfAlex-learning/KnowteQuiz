<template>
  <div class="flex-1 flex flex-col relative">
    <!-- Search bar -->
    <div
      v-if="showSearch"
      class="flex items-center gap-2 px-4 py-2 border-b border-[var(--border-default)] bg-[var(--bg-elevated)]"
    >
      <input
        ref="searchInput"
        v-model="searchQuery"
        type="text"
        :placeholder="t('reader.find_in_note')"
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

    <div class="flex-1 flex">
      <!-- TOC sidebar -->
      <aside
        v-if="showToc && tocHeadings.length > 0"
        class="w-48 shrink-0 overflow-y-auto border-r border-[var(--border-default)] bg-[var(--bg-elevated)] p-3"
      >
        <div class="flex items-center justify-between mb-2">
          <span class="text-[11px] font-medium text-[var(--text-secondary)]">{{ t('reader.outline') }}</span>
          <button
            class="text-[11px] text-[var(--text-muted)] hover:text-[var(--text-primary)]"
            @click="showToc = false"
          >
            ✕
          </button>
        </div>
        <nav>
          <a
            v-for="h in tocHeadings"
            :key="h.id"
            class="block truncate py-0.5 text-[11px] leading-relaxed cursor-pointer transition-colors"
            :class="{
              'text-[var(--text-secondary)] hover:text-[var(--text-primary)]': true,
              'pl-0': h.level === 1,
              'pl-3': h.level === 2,
              'pl-5': h.level === 3,
              'pl-7': h.level >= 4,
            }"
            @click.prevent="scrollToHeading(h.id)"
          >
            {{ h.text }}
          </a>
        </nav>
      </aside>

      <!-- Main content -->
      <div class="flex-1 overflow-y-auto p-6">
        <!-- Loading state -->
        <LoadingSpinner v-if="readerStore.isLoading" :label="t('reader.loading')" overlay />

        <!-- Error state -->
        <div v-else-if="readerStore.error" class="flex flex-col items-center justify-center h-full text-center">
          <svg
            class="w-12 h-12 text-[var(--color-error)] mb-4"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
          >
            <circle cx="12" cy="12" r="10" />
            <path d="M12 8v4m0 4h.01" />
          </svg>
          <p class="text-[var(--text-sm)] text-[var(--color-error)]">{{ readerStore.error }}</p>
        </div>

        <!-- Rendered markdown -->
        <article
          v-else-if="readerStore.currentNote"
          ref="articleRef"
          class="markdown-body max-w-none"
          v-html="renderedHtml"
        />

        <!-- Empty state -->
        <EmptyState v-else />
      </div>
    </div>

    <!-- TOC toggle button -->
    <button
      v-if="tocHeadings.length > 0 && !showToc"
      class="absolute right-3 bottom-3 w-8 h-8 rounded-full bg-[var(--accent-purple)]/20 text-[var(--accent-purple)] flex items-center justify-center text-xs font-medium shadow hover:bg-[var(--accent-purple)]/30 transition-colors z-10"
      :title="t('reader.outline')"
      @click="showToc = true"
    >
      ☰
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import MarkdownIt from 'markdown-it';
import mk from '@traptitech/markdown-it-katex';
import hljs from 'highlight.js';
import 'katex/dist/katex.min.css';
import { useReaderStore } from '@/stores/reader';
import { convertFileSrc, isTauri } from '@/services/tauri';
import { renderMarkdownWithFallback, uniqueHeadingId } from '@/utils/markdown';
import { extractHeadings, type TocHeading } from '@/utils/markdown';
import { configureMarkdownAssetRenderer, markdownWebAssetUrl } from '@/utils/markdownAssets';
import { useI18n } from '@/composables/useI18n';
import EmptyState from './EmptyState.vue';
import LoadingSpinner from '@/components/common/LoadingSpinner.vue';

const readerStore = useReaderStore();
const { t } = useI18n();

const md = new MarkdownIt({
  html: false,
  linkify: true,
  typographer: true,
  highlight(str: string, lang: string): string {
    if (lang && hljs.getLanguage(lang)) {
      try {
        return `<pre class="hljs"><code>${hljs.highlight(str, { language: lang }).value}</code></pre>`;
      } catch {
        // fall through
      }
    }
    return `<pre class="hljs"><code>${md.utils.escapeHtml(str)}</code></pre>`;
  },
});

md.use(mk);
configureMarkdownAssetRenderer(
  md,
  () => readerStore.currentNote?.path,
  (path) => (isTauri() ? convertFileSrc(path) : markdownWebAssetUrl(path))
);

let renderHeadingCounts = new Map<string, number>();
md.renderer.rules.heading_open = function (tokens, idx) {
  const token = tokens[idx];
  const level = token.tag;
  const nextToken = tokens[idx + 1];
  const text = nextToken?.type === 'inline' ? nextToken.content || '' : '';
  const id = uniqueHeadingId(text, renderHeadingCounts);
  return `<${level} id="${id}">`;
};

const renderedHtml = computed(() => {
  if (!readerStore.currentNote?.content) return '';
  return renderMarkdownWithFallback(readerStore.currentNote.content, (source) => {
    renderHeadingCounts = new Map<string, number>();
    return md.render(source);
  });
});

// TOC outline
const showToc = ref(false);
const tocHeadings = computed<TocHeading[]>(() => {
  if (!readerStore.currentNote?.content) return [];
  return extractHeadings(readerStore.currentNote.content);
});

function scrollToHeading(id: string) {
  const el = document.getElementById(id);
  if (el) {
    el.scrollIntoView({ behavior: 'smooth', block: 'start' });
  }
}

// In-note search: DOM-based highlighting over the rendered article. The raw
// HTML stays the source of truth; matches are <mark> nodes injected into the
// live DOM and removed by restoring the original innerHTML.
const showSearch = ref(false);
const searchQuery = ref('');
const searchInput = ref<HTMLInputElement | null>(null);
const articleRef = ref<HTMLElement | null>(null);
const matchInfo = ref('');
const searchMatches = ref<HTMLElement[]>([]);
const activeMatchIndex = ref(-1);
let searchTimer: ReturnType<typeof setTimeout> | null = null;

function handleSearchInput() {
  if (searchTimer) clearTimeout(searchTimer);
  searchTimer = setTimeout(() => {
    doSearch();
  }, 150);
}

function doSearch() {
  const query = searchQuery.value.trim().toLowerCase();
  if (!query) {
    clearFind();
    return;
  }
  const container = articleRef.value;
  if (!container) return;

  // Start from the pristine render so repeated searches never nest marks.
  container.innerHTML = renderedHtml.value;
  searchMatches.value = [];
  activeMatchIndex.value = -1;

  const walker = document.createTreeWalker(container, NodeFilter.SHOW_TEXT, {
    acceptNode(node: Node): number {
      const parent = (node as Text).parentElement;
      if (parent && (parent.tagName === 'SCRIPT' || parent.tagName === 'STYLE')) {
        return NodeFilter.FILTER_REJECT;
      }
      return node.nodeValue && node.nodeValue.toLowerCase().includes(query)
        ? NodeFilter.FILTER_ACCEPT
        : NodeFilter.FILTER_REJECT;
    },
  });

  const textNodes: Text[] = [];
  let current = walker.nextNode();
  while (current) {
    textNodes.push(current as Text);
    current = walker.nextNode();
  }

  for (const node of textNodes) {
    const text = node.nodeValue ?? '';
    const lower = text.toLowerCase();
    const fragment = document.createDocumentFragment();
    let cursor = 0;
    let start = lower.indexOf(query);
    while (start >= 0) {
      if (start > cursor) {
        fragment.appendChild(document.createTextNode(text.slice(cursor, start)));
      }
      const mark = document.createElement('mark');
      mark.setAttribute('data-search-hit', '');
      mark.textContent = text.slice(start, start + query.length);
      fragment.appendChild(mark);
      searchMatches.value.push(mark);
      cursor = start + query.length;
      start = lower.indexOf(query, cursor);
    }
    if (cursor < text.length) {
      fragment.appendChild(document.createTextNode(text.slice(cursor)));
    }
    node.parentNode?.replaceChild(fragment, node);
  }

  if (searchMatches.value.length > 0) {
    setActiveMatch(0);
  } else {
    matchInfo.value = '0/0';
  }
}

function setActiveMatch(index: number) {
  searchMatches.value.forEach((mark, i) => {
    if (i === index) {
      mark.setAttribute('data-search-active', '');
    } else {
      mark.removeAttribute('data-search-active');
    }
  });
  activeMatchIndex.value = index;
  matchInfo.value = `${index + 1}/${searchMatches.value.length}`;
  // Optional call: jsdom test environments don't implement scrollIntoView.
  searchMatches.value[index]?.scrollIntoView?.({ behavior: 'smooth', block: 'center' });
}

function findNext() {
  if (searchMatches.value.length === 0) return;
  setActiveMatch((activeMatchIndex.value + 1) % searchMatches.value.length);
}

function findPrev() {
  if (searchMatches.value.length === 0) return;
  setActiveMatch((activeMatchIndex.value - 1 + searchMatches.value.length) % searchMatches.value.length);
}

function clearFind() {
  const container = articleRef.value;
  if (container) {
    container.innerHTML = renderedHtml.value;
  }
  searchMatches.value = [];
  activeMatchIndex.value = -1;
  matchInfo.value = '';
}

function closeSearch() {
  showSearch.value = false;
  searchQuery.value = '';
  clearFind();
}

async function openSearch() {
  showSearch.value = true;
  await nextTick();
  searchInput.value?.focus();
  if (searchQuery.value) {
    doSearch();
  }
}

function handleKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key === 'f') {
    e.preventDefault();
    openSearch();
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown);
});

// Re-run the active search when the note content changes under it.
watch(renderedHtml, () => {
  if (!showSearch.value || !searchQuery.value.trim()) return;
  void nextTick(doSearch);
});
</script>

<style scoped>
:deep(mark[data-search-hit]) {
  background: color-mix(in srgb, var(--accent-purple) 35%, transparent);
  color: inherit;
  border-radius: 2px;
}

:deep(mark[data-search-active]) {
  background: var(--accent-purple);
  color: var(--bg-base);
}
</style>
