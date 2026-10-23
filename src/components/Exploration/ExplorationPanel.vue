<template>
  <div class="h-full flex flex-col bg-[var(--bg-sidebar)]" data-glass-fill>
    <!-- Status -->
    <div class="px-3 py-2 border-b border-[var(--border-subtle)] space-y-1.5">
      <div class="flex items-center justify-between gap-2">
        <h2 class="text-[var(--text-xs)] font-semibold uppercase tracking-wider text-[var(--text-muted)]">
          {{ t('exploration.status_title') }}
        </h2>
        <span
          class="px-2 py-0.5 rounded-full text-[10px] font-semibold uppercase tracking-wider bg-[var(--bg-elevated)] text-[var(--text-muted)]"
        >
          {{ t('exploration.status_initialise') }}
        </span>
      </div>
      <p class="text-[var(--text-xs)] text-[var(--text-faint)]">
        {{ t('exploration.status_idle') }} · {{ t('exploration.status_hint') }}
      </p>
    </div>

    <!-- Whiteboard -->
    <div class="flex-1 flex flex-col min-h-0">
      <h2 class="px-3 pt-3 pb-1 text-[var(--text-xs)] font-semibold uppercase tracking-wider text-[var(--text-muted)]">
        {{ t('exploration.whiteboard') }}
      </h2>
      <div class="flex-1 min-h-0 overflow-auto px-3 pb-4">
        <div
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
import { useI18n } from '@/composables/useI18n';

const { t } = useI18n();
</script>
