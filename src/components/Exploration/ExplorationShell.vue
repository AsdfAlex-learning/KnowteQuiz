<template>
  <div class="h-full flex flex-col">
    <!-- Header -->
    <header class="px-6 py-5 border-b border-[var(--border-subtle)] flex items-start justify-between gap-4">
      <div class="min-w-0">
        <h1 class="text-[var(--text-lg)] font-semibold text-[var(--text-primary)]">
          {{ t('exploration.title') }}
        </h1>
        <p class="mt-1 text-[var(--text-sm)] text-[var(--text-muted)] max-w-xl leading-relaxed">
          {{ t('exploration.intro') }}
        </p>
      </div>
      <span
        class="shrink-0 px-2.5 py-1 rounded-full text-[var(--text-xs)] font-medium"
        :class="
          explorationStore.canStart
            ? 'bg-[var(--accent-purple)]/15 text-[var(--accent-purple)]'
            : 'bg-[var(--bg-elevated)] text-[var(--text-muted)]'
        "
      >
        {{
          explorationStore.canStart
            ? t('exploration.selected_count', { count: explorationStore.selectedCount })
            : t('exploration.none_selected')
        }}
      </span>
    </header>

    <!-- Onboarding -->
    <div class="flex-1 flex items-center justify-center px-6 py-8">
      <div class="w-full max-w-md space-y-6">
        <ol class="space-y-4">
          <li v-for="(step, index) in steps" :key="index" class="flex items-start gap-3">
            <span
              class="shrink-0 w-6 h-6 rounded-full flex items-center justify-center text-[var(--text-xs)] font-semibold mt-0.5"
              :class="
                index === 0 && explorationStore.canStart
                  ? 'bg-[var(--accent-purple)] text-[var(--bg-base)]'
                  : 'bg-[var(--bg-elevated)] text-[var(--text-muted)]'
              "
            >
              {{ index + 1 }}
            </span>
            <p class="text-[var(--text-sm)] text-[var(--text-secondary)] leading-relaxed">
              {{ step }}
            </p>
          </li>
        </ol>

        <div v-if="!explorerStore.rootPath" class="pt-2 flex items-center gap-3">
          <button
            class="px-4 py-2 rounded-[var(--radius-md)] bg-[var(--accent-purple)] text-[var(--bg-base)] font-medium text-[var(--text-sm)] hover:bg-[var(--accent-purple)]/85 transition-colors"
            @click="explorerStore.chooseFolder()"
          >
            {{ t('exploration.open_folder') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useExplorationStore } from '@/stores/exploration';
import { useExplorerStore } from '@/stores/explorer';
import { useI18n } from '@/composables/useI18n';

const explorationStore = useExplorationStore();
const explorerStore = useExplorerStore();
const { t } = useI18n();

const steps = computed(() => [t('exploration.step_select'), t('exploration.step_graph'), t('exploration.step_learn')]);
</script>
