<template>
  <div class="h-full flex flex-col bg-[var(--bg-sidebar)]" data-glass-fill>
    <!-- Status -->
    <div class="px-3 py-2 border-b border-[var(--border-subtle)] space-y-1.5">
      <div class="flex items-center justify-between gap-2">
        <h2 class="text-[var(--text-xs)] font-semibold uppercase tracking-wider text-[var(--text-muted)]">
          {{ t('exploration.status_title') }}
        </h2>
        <span
          class="px-2 py-0.5 rounded-full text-[10px] font-semibold uppercase tracking-wider"
          :class="statusBadgeClass"
        >
          {{ statusLabel }}
        </span>
      </div>
      <div class="flex items-center justify-between gap-2 text-[var(--text-xs)] text-[var(--text-faint)]">
        <span>
          {{
            t('exploration.coverage', {
              covered: explorationStore.coverage.covered,
              total: explorationStore.coverage.total,
            })
          }}
        </span>
        <span v-if="session">
          {{ t('exploration.session_sources', { count: session.source_paths.length }) }}
        </span>
        <span v-else>{{ t('exploration.status_hint') }}</span>
      </div>
    </div>

    <!-- Whiteboard -->
    <div class="flex-1 flex flex-col min-h-0">
      <h2 class="px-3 pt-3 pb-1 text-[var(--text-xs)] font-semibold uppercase tracking-wider text-[var(--text-muted)]">
        {{ t('exploration.whiteboard') }}
      </h2>
      <div class="flex-1 min-h-0 overflow-auto px-3 pb-4">
        <div v-if="whiteboard" class="space-y-2">
          <p v-if="whiteboard.title" class="text-[var(--text-xs)] font-medium text-[var(--text-secondary)]">
            {{ whiteboard.title }}
          </p>
          <div
            class="quiz-md text-[var(--text-sm)] text-[var(--text-primary)] leading-relaxed"
            v-html="renderedWhiteboard"
          ></div>
        </div>
        <div
          v-else
          class="h-full min-h-[140px] flex flex-col items-center justify-center gap-2 rounded-[var(--radius-md)] border border-dashed border-[var(--border-default)] text-center px-4 py-6"
        >
          <svg
            class="w-6 h-6 text-[var(--text-faint)]"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.3"
          >
            <rect x="3" y="4" width="18" height="13" rx="1.5" />
            <path d="M12 17v3M8 20h8" />
          </svg>
          <p class="text-[var(--text-xs)] text-[var(--text-faint)] leading-relaxed max-w-[220px]">
            {{ t('exploration.whiteboard_empty') }}
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useExplorationStore } from '@/stores/exploration';
import { useI18n } from '@/composables/useI18n';
import { renderQuizMarkdown } from '@/utils/markdown';

const explorationStore = useExplorationStore();
const { t } = useI18n();

const session = computed(() => explorationStore.session);
const whiteboard = computed(() => explorationStore.currentWhiteboard);
const renderedWhiteboard = computed(() => (whiteboard.value ? renderQuizMarkdown(whiteboard.value.content_md) : ''));

const statusLabel = computed(() => {
  switch (session.value?.status) {
    case 'initializing':
      return t('exploration.status_initialise');
    case 'awaiting_domain_confirm':
      return t('exploration.status_awaiting_domains');
    case 'active':
      return t('exploration.status_active');
    case 'completed':
      return t('exploration.status_completed');
    default:
      return t('exploration.status_idle');
  }
});

const statusBadgeClass = computed(() => {
  switch (session.value?.status) {
    case 'active':
      return 'bg-[var(--color-success)]/15 text-[var(--color-success)]';
    case 'completed':
    case 'initializing':
      return 'bg-[var(--accent-purple)]/15 text-[var(--accent-purple)]';
    case 'awaiting_domain_confirm':
      return 'bg-[var(--color-warning)]/15 text-[var(--color-warning)]';
    default:
      return 'bg-[var(--bg-elevated)] text-[var(--text-muted)]';
  }
});
</script>
