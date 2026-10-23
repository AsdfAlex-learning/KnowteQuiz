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

      <!-- Learning loop -->
      <div v-else-if="explorationStore.hasGraph" class="max-w-2xl mx-auto space-y-6">
        <div
          v-if="isCompleted"
          class="rounded-[var(--radius-md)] border border-[var(--color-success)]/30 bg-[var(--color-success)]/10 px-4 py-3 space-y-1.5"
        >
          <h2 class="text-[var(--text-sm)] font-semibold text-[var(--color-success)]">
            {{ t('exploration.completed_title') }}
          </h2>
          <p class="text-[var(--text-xs)] text-[var(--text-secondary)]">
            {{
              t('exploration.completed_summary', {
                covered: explorationStore.coverage.covered,
                total: explorationStore.coverage.total,
              })
            }}
          </p>
          <div v-if="uncovered.length">
            <p class="text-[11px] font-medium text-[var(--text-muted)]">
              {{ t('exploration.uncovered_label') }}
            </p>
            <ul class="mt-1 space-y-0.5">
              <li v-for="concept in uncovered" :key="concept.id" class="text-[11px] text-[var(--text-faint)]">
                · {{ concept.title }}
              </li>
            </ul>
          </div>
        </div>

        <div class="space-y-5">
          <ExplorationTurn
            v-for="(group, index) in answeredGroups"
            :key="index"
            :group="group"
            :concept-title="conceptTitle(group.concept_id)"
          />
        </div>

        <div
          v-if="currentGroup"
          class="space-y-3 rounded-[var(--radius-md)] border border-[var(--border-default)] bg-[var(--bg-elevated)] px-3 py-3"
        >
          <div class="flex items-center gap-2">
            <p class="text-[10px] font-semibold uppercase tracking-wider text-[var(--text-faint)]">
              {{ t('exploration.current_concept') }} · {{ conceptTitle(currentGroup.concept_id) }}
            </p>
            <span
              v-if="currentGroup.question.hint_used"
              class="px-1.5 py-0.5 rounded text-[10px] font-semibold uppercase tracking-wider bg-[var(--color-warning)]/15 text-[var(--color-warning)]"
            >
              {{ t('exploration.hint_label') }}
            </span>
          </div>
          <div
            class="quiz-md text-[var(--text-sm)] text-[var(--text-primary)] leading-relaxed"
            v-html="currentQuestionHtml"
          ></div>
          <ExplorationComposer :busy="explorationStore.thinking" />
        </div>

        <div v-else-if="session.status === 'active'" class="text-center">
          <button
            class="px-4 py-2 rounded-[var(--radius-md)] bg-[var(--accent-purple)] text-[var(--bg-base)] font-medium text-[var(--text-sm)] hover:bg-[var(--accent-purple)]/85 transition-colors disabled:opacity-50"
            :disabled="explorationStore.thinking"
            @click="explorationStore.requestNextQuestion()"
          >
            {{ explorationStore.thinking ? t('exploration.thinking') : t('exploration.continue') }}
          </button>
        </div>

        <p v-if="explorationStore.error" class="text-[var(--text-xs)] text-[var(--color-error)]">
          {{ explorationStore.error }}
        </p>

        <details class="rounded-[var(--radius-md)] border border-[var(--border-subtle)]">
          <summary
            class="cursor-pointer px-3 py-2 text-[var(--text-xs)] font-medium text-[var(--text-muted)] hover:text-[var(--text-primary)]"
          >
            {{
              t('exploration.graph_summary', {
                concepts: session.graph.concepts.length,
                domains: session.graph.domains.length,
              })
            }}
          </summary>
          <div class="px-3 pb-3">
            <ExplorationGraph :graph="session.graph" />
          </div>
        </details>
      </div>

      <!-- Initializing -->
      <div v-else class="h-full flex items-center justify-center">
        <div class="text-center space-y-3 max-w-sm">
          <template v-if="(explorationStore.streaming || explorationStore.isInitializing) && !explorationStore.error">
            <span
              class="inline-block w-5 h-5 border-2 border-[var(--border-focus)] border-t-transparent rounded-full animate-spin"
            />
            <p class="text-[var(--text-sm)] text-[var(--text-secondary)]">{{ phaseLabel }}</p>
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
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useExplorationStore } from '@/stores/exploration';
import { useI18n } from '@/composables/useI18n';
import { renderQuizMarkdown } from '@/utils/markdown';
import type { Concept } from '@/types/exploration';
import DomainConfirm from './DomainConfirm.vue';
import ExplorationGraph from './ExplorationGraph.vue';
import ExplorationTurn from './ExplorationTurn.vue';
import ExplorationComposer from './ExplorationComposer.vue';

const explorationStore = useExplorationStore();
const { t } = useI18n();

const session = computed(() => explorationStore.session);
const isCompleted = computed(() => session.value?.status === 'completed');

const answeredGroups = computed(() => {
  const current = session.value;
  if (!current) return [];
  return current.groups.slice(0, current.active_group);
});

const currentGroup = computed(() => explorationStore.currentGroup);

const currentQuestionHtml = computed(() =>
  currentGroup.value ? renderQuizMarkdown(currentGroup.value.question.prompt_md) : ''
);

const uncovered = computed<Concept[]>(() => {
  const current = session.value;
  if (!current) return [];
  const covered = new Set<string>();
  for (const group of current.groups.slice(0, current.active_group)) {
    for (const id of group.covered_after) covered.add(id);
  }
  return current.graph.concepts.filter((concept) => !covered.has(concept.id));
});

function conceptTitle(id: string): string {
  return session.value?.graph.concepts.find((concept) => concept.id === id)?.title ?? '';
}

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
