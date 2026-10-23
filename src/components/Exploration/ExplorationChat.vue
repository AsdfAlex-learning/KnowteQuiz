<template>
  <div v-if="session" class="h-full flex flex-col">
    <header class="px-6 py-4 border-b border-[var(--border-subtle)] flex items-start justify-between gap-4">
      <div class="min-w-0">
        <h1 class="truncate text-[var(--text-lg)] font-semibold text-[var(--text-primary)]">
          {{ session.title }}
        </h1>
        <p class="mt-1 text-[var(--text-xs)] text-[var(--text-muted)]">
          {{ t('exploration.session_sources', { count: session.source_paths.length }) }}
        </p>
      </div>
      <button
        class="shrink-0 px-2.5 py-1 rounded-[var(--radius-sm)] text-[var(--text-xs)] text-[var(--text-muted)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)] transition-colors"
        @click="explorationStore.closeSession()"
      >
        {{ t('exploration.exit_session') }}
      </button>
    </header>

    <div class="flex-1 flex items-center justify-center px-6 py-8">
      <div v-if="explorationStore.isInitializing" class="text-center space-y-3 max-w-sm">
        <span
          class="inline-block w-5 h-5 border-2 border-[var(--border-focus)] border-t-transparent rounded-full animate-spin"
        />
        <p class="text-[var(--text-sm)] text-[var(--text-secondary)]">
          {{ t('exploration.initializing') }}
        </p>
        <p class="text-[var(--text-xs)] text-[var(--text-faint)] leading-relaxed">
          {{ t('exploration.initializing_hint') }}
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useExplorationStore } from '@/stores/exploration';
import { useI18n } from '@/composables/useI18n';

const explorationStore = useExplorationStore();
const { t } = useI18n();

const session = computed(() => explorationStore.session);
</script>
