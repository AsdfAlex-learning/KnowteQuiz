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

    <div class="flex-1 overflow-auto px-6 py-6">
      <!-- Domain confirmation blocks initialization -->
      <DomainConfirm
        v-if="explorationStore.pendingDomains"
        :domains="explorationStore.pendingDomains"
        :confirming="explorationStore.streaming"
        @confirm="explorationStore.confirmDomains"
      />

      <!-- Knowledge map ready -->
      <div v-else-if="explorationStore.hasGraph" class="max-w-2xl mx-auto space-y-4">
        <div class="flex items-center justify-between gap-3">
          <div>
            <h2 class="text-[var(--text-sm)] font-semibold text-[var(--text-primary)]">
              {{ t('exploration.graph_ready') }}
            </h2>
            <p class="mt-0.5 text-[var(--text-xs)] text-[var(--text-muted)]">
              {{
                t('exploration.graph_summary', {
                  concepts: session.graph.concepts.length,
                  domains: session.graph.domains.length,
                })
              }}
            </p>
          </div>
        </div>
        <div class="rounded-[var(--radius-md)] border border-[var(--border-default)] bg-[var(--bg-elevated)] p-3">
          <ExplorationGraph :graph="session.graph" />
        </div>
        <p class="text-[var(--text-xs)] text-[var(--text-faint)] text-center">
          {{ t('exploration.graph_next') }}
        </p>
      </div>

      <!-- Initializing -->
      <div v-else class="h-full flex items-center justify-center">
        <div class="text-center space-y-3 max-w-sm">
          <template v-if="(explorationStore.streaming || explorationStore.isInitializing) && !explorationStore.error">
            <span
              class="inline-block w-5 h-5 border-2 border-[var(--border-focus)] border-t-transparent rounded-full animate-spin"
            />
            <p class="text-[var(--text-sm)] text-[var(--text-secondary)]">
              {{ phaseLabel }}
            </p>
            <p class="text-[var(--text-xs)] text-[var(--text-faint)] leading-relaxed">
              {{ t('exploration.initializing_hint') }}
            </p>
          </template>
          <template v-else>
            <p class="text-[var(--text-sm)] text-[var(--text-secondary)]">
              {{ explorationStore.error || t('exploration.initializing_hint') }}
            </p>
            <button
              v-if="explorationStore.error"
              class="px-4 py-2 rounded-[var(--radius-md)] bg-[var(--accent-purple)] text-[var(--bg-base)] font-medium text-[var(--text-sm)] hover:bg-[var(--accent-purple)]/85 transition-colors"
              @click="explorationStore.retryInitialization()"
            >
              {{ t('exploration.retry_initialization') }}
            </button>
          </template>
        </div>
      </div>
    </div>

    <div v-if="explorationStore.error && explorationStore.hasGraph" class="px-6 pb-3">
      <p class="text-[var(--text-xs)] text-[var(--color-error)]">{{ explorationStore.error }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useExplorationStore } from '@/stores/exploration';
import { useI18n } from '@/composables/useI18n';
import DomainConfirm from './DomainConfirm.vue';
import ExplorationGraph from './ExplorationGraph.vue';

const explorationStore = useExplorationStore();
const { t } = useI18n();

const session = computed(() => explorationStore.session);

const phaseLabel = computed(() => {
  const phase = explorationStore.phase;
  if (!phase) return t('exploration.initializing');
  if (phase === 'reading') return t('exploration.phase_reading');
  if (phase === 'merging') return t('exploration.phase_merging');
  if (phase.startsWith('summarizing:')) {
    const [current, total] = phase.slice('summarizing:'.length).split('/');
    return t('exploration.phase_summarizing', { current, total });
  }
  return t('exploration.initializing');
});
</script>
