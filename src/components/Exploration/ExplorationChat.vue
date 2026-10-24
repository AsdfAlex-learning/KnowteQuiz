<template>
  <div v-if="session" class="h-full flex flex-col">
    <header class="px-6 py-4 border-b border-[var(--border-subtle)] flex items-start justify-between gap-4">
      <div class="min-w-0">
        <h1 class="truncate text-[var(--text-lg)] font-semibold text-[var(--text-primary)]">
          {{ session.title }}
        </h1>
        <p class="mt-1 text-[var(--text-xs)] text-[var(--text-muted)] flex items-center gap-2">
          {{ t('exploration.session_sources', { count: session.source_paths.length }) }}
          <span
            v-if="explorationStore.autoResumed"
            class="px-1.5 py-0.5 rounded-full bg-[var(--color-success)]/15 text-[var(--color-success)] text-[10px]"
          >
            {{ t('exploration.resume_banner') }}
          </span>
        </p>
      </div>
      <button
        class="shrink-0 px-2.5 py-1 rounded-[var(--radius-sm)] text-[var(--text-xs)] text-[var(--text-muted)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)] transition-colors"
        @click="explorationStore.closeSession()"
      >
        {{ t('exploration.exit_session') }}
      </button>
    </header>

    <div
      v-if="explorationStore.isReadOnly"
      class="px-6 py-2 border-b border-[var(--border-subtle)] bg-[var(--color-warning)]/10 flex items-center justify-between gap-3"
    >
      <p class="text-[var(--text-xs)] text-[var(--text-muted)]">
        {{ t('exploration.readonly_badge') }} · {{ t('exploration.readonly_hint') }}
      </p>
      <button
        class="shrink-0 px-3 py-1 rounded-[var(--radius-sm)] bg-[var(--accent-purple)] text-[var(--bg-base)] text-[var(--text-xs)] font-medium hover:bg-[var(--accent-purple)]/85 transition-colors"
        @click="explorationStore.continueSession()"
      >
        {{ t('exploration.continue_learning') }}
      </button>
    </div>

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
          v-if="explorationStore.isReplaying"
          class="flex flex-wrap items-center gap-2 rounded-[var(--radius-md)] border border-[var(--border-default)] bg-[var(--bg-elevated)] px-3 py-2"
        >
          <button
            class="px-2 py-1 rounded-[var(--radius-sm)] text-[var(--text-xs)] text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] disabled:opacity-40"
            :title="t('exploration.replay_prev')"
            :disabled="explorationStore.displayedGroupCount === 0"
            @click="explorationStore.replayPrev()"
          >
            ◀
          </button>
          <span class="text-[11px] tabular-nums text-[var(--text-muted)]">
            {{ explorationStore.displayedGroupCount }} / {{ session.active_group }}
          </span>
          <button
            class="px-2 py-1 rounded-[var(--radius-sm)] text-[var(--text-xs)] text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] disabled:opacity-40"
            :title="t('exploration.replay_next')"
            :disabled="explorationStore.displayedGroupCount >= session.active_group"
            @click="explorationStore.replayNext()"
          >
            ▶
          </button>
          <label class="ml-auto flex items-center gap-1.5 text-[11px] text-[var(--text-muted)]">
            <input
              type="checkbox"
              :checked="explorationStore.skipAsides"
              @change="explorationStore.toggleSkipAsides()"
            />
            {{ t('exploration.replay_skip_asides') }}
          </label>
          <button
            class="px-2 py-1 rounded-[var(--radius-sm)] text-[11px] text-[var(--text-muted)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]"
            @click="explorationStore.stopReplay()"
          >
            {{ t('exploration.replay_exit') }}
          </button>
        </div>

        <div
          v-if="isCompleted"
          class="rounded-[var(--radius-md)] border border-[var(--color-success)]/30 bg-[var(--color-success)]/10 px-4 py-3 space-y-1.5"
        >
          <h2 class="text-[var(--text-sm)] font-semibold text-[var(--color-success)]">
            {{ t('exploration.completed_title') }}
          </h2>
          <p v-if="session.completed_early" class="text-[var(--text-xs)] text-[var(--color-warning)]">
            {{ t('exploration.completed_early') }}
          </p>
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
          <div class="flex flex-wrap items-center gap-2 pt-1">
            <button
              class="px-3 py-1.5 rounded-[var(--radius-sm)] bg-[var(--accent-purple)] text-[var(--bg-base)] text-[var(--text-xs)] font-medium hover:bg-[var(--accent-purple)]/85 transition-colors"
              @click="explorationStore.exportSession()"
            >
              {{ t('exploration.export') }}
            </button>
            <button
              class="px-3 py-1.5 rounded-[var(--radius-sm)] border border-[var(--border-default)] text-[var(--text-xs)] text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] transition-colors"
              @click="showVault = !showVault"
            >
              {{ t('exploration.save_to_vault') }}
            </button>
            <button
              class="px-3 py-1.5 rounded-[var(--radius-sm)] border border-[var(--border-default)] text-[var(--text-xs)] text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] transition-colors"
              @click="explorationStore.startReplay()"
            >
              {{ t('exploration.replay') }}
            </button>
          </div>
          <div v-if="showVault" class="flex items-center gap-2">
            <input
              v-model="vaultName"
              type="text"
              :placeholder="t('exploration.vault_file_name')"
              class="flex-1 px-2 py-1 text-[var(--text-xs)] bg-[var(--bg-base)] text-[var(--text-primary)] border border-[var(--border-default)] rounded focus:border-[var(--border-focus)] focus:outline-none"
            />
            <button
              class="px-3 py-1 rounded-[var(--radius-sm)] bg-[var(--accent-purple)] text-[var(--bg-base)] text-[var(--text-xs)] font-medium disabled:opacity-50"
              :disabled="!vaultName.trim() || savingVault"
              @click="saveVault"
            >
              {{ t('exploration.save') }}
            </button>
          </div>
          <p v-if="explorationStore.lastExport" class="text-[11px] text-[var(--text-faint)] break-all">
            {{ t('exploration.export_saved') }}: {{ explorationStore.lastExport.path }}
          </p>
          <p v-if="explorationStore.lastVaultPath" class="text-[11px] text-[var(--color-success)] break-all">
            {{ t('exploration.vault_saved') }}: {{ explorationStore.lastVaultPath }}
          </p>
        </div>

        <div class="space-y-5">
          <template v-for="(group, index) in answeredGroups" :key="index">
            <ExplorationTurn :group="group" :concept-title="conceptTitle(group.concept_id)" />
            <AsideBubble v-for="aside in asidesAt(index)" :key="aside.id" :aside="aside" />
          </template>
        </div>

        <template v-if="!currentGroup">
          <AsideBubble v-for="aside in asidesAt(activeIndex)" :key="aside.id" :aside="aside" />
        </template>

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
          <AsideBubble v-for="aside in asidesAt(activeIndex)" :key="aside.id" :aside="aside" />
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
import { computed, ref } from 'vue';
import { useExplorationStore } from '@/stores/exploration';
import { useI18n } from '@/composables/useI18n';
import { renderQuizMarkdown } from '@/utils/markdown';
import type { Concept } from '@/types/exploration';
import DomainConfirm from './DomainConfirm.vue';
import ExplorationGraph from './ExplorationGraph.vue';
import ExplorationTurn from './ExplorationTurn.vue';
import ExplorationComposer from './ExplorationComposer.vue';
import AsideBubble from './AsideBubble.vue';

const explorationStore = useExplorationStore();
const { t } = useI18n();
const showVault = ref(false);
const vaultName = ref('');
const savingVault = ref(false);

const session = computed(() => explorationStore.session);
const isCompleted = computed(() => session.value?.status === 'completed' && !explorationStore.isReplaying);

const answeredGroups = computed(() => {
  const current = session.value;
  if (!current) return [];
  return current.groups.slice(0, explorationStore.displayedGroupCount);
});

const currentGroup = computed(() => (explorationStore.isReplaying ? null : explorationStore.currentGroup));

const activeIndex = computed(() => session.value?.active_group ?? 0);
const asides = computed(() => session.value?.asides ?? []);

function asidesAt(index: number) {
  if (explorationStore.isReplaying && explorationStore.skipAsides) return [];
  return asides.value.filter((aside) => aside.group_index === index);
}

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

async function saveVault() {
  savingVault.value = true;
  try {
    if (await explorationStore.saveToVault(vaultName.value)) {
      vaultName.value = '';
      showVault.value = false;
    }
  } finally {
    savingVault.value = false;
  }
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
