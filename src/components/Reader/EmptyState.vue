<template>
  <div class="flex flex-col items-center justify-center h-full text-center px-8">
    <!-- Decorative icon -->
    <div class="w-20 h-20 rounded-2xl bg-[var(--accent-purple)]/10 flex items-center justify-center mb-6">
      <svg class="w-10 h-10 text-[var(--accent-purple)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M4 4h16v16H4V4z" stroke-dasharray="4 2" />
        <path d="M8 8h8M8 12h6M8 16h4" />
      </svg>
    </div>

    <h2 class="text-[var(--text-xl)] font-semibold text-[var(--text-primary)] mb-2">
      KnowteQuiz
    </h2>
    <p class="text-[var(--text-sm)] text-[var(--text-muted)] max-w-xs leading-relaxed">
      Select a note from the explorer to start reading, or open a folder to browse your markdown files.
    </p>

    <div v-if="!explorerStore.rootPath" class="mt-6">
      <button
        class="px-5 py-2.5 rounded-[var(--radius-md)] bg-[var(--accent-purple)] text-[var(--bg-base)] font-medium text-[var(--text-sm)] hover:bg-[var(--accent-purple)]/85 transition-colors"
        @click="explorerStore.chooseFolder()"
      >
        Open a Folder
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useExplorerStore } from '@/stores/explorer'

const explorerStore = useExplorerStore()
</script>