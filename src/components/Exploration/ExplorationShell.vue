<template>
  <ExplorationChat v-if="explorationStore.session" />
  <div v-else class="h-full flex flex-col">
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

        <div class="space-y-2">
          <div class="flex items-center justify-between">
            <label class="text-[var(--text-sm)] font-medium text-[var(--text-primary)]">
              {{ t('exploration.baseline') }}
            </label>
            <span class="text-[var(--text-xs)] font-mono text-[var(--text-muted)]">
              {{ baseline }} · {{ baselineLabel }}
            </span>
          </div>
          <input
            type="range"
            min="0"
            max="100"
            step="5"
            :value="baseline"
            class="w-full"
            @input="onBaselineInput"
            @change="persistBaseline"
          />
          <div class="flex justify-between text-[10px] text-[var(--text-faint)]">
            <span>{{ t('exploration.baseline_zero') }}</span>
            <span>{{ t('exploration.baseline_advanced') }}</span>
          </div>
          <p class="text-[11px] text-[var(--text-faint)]">
            {{ t('exploration.baseline_hint') }}
          </p>
        </div>

        <div class="pt-2 flex flex-col gap-3">
          <div class="flex items-center gap-3">
            <button
              class="px-4 py-2 rounded-[var(--radius-md)] bg-[var(--accent-purple)] text-[var(--bg-base)] font-medium text-[var(--text-sm)] hover:bg-[var(--accent-purple)]/85 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              :disabled="!explorationStore.canStart"
              @click="start"
            >
              {{ explorationStore.creating ? t('exploration.starting') : t('exploration.start') }}
            </button>
            <button
              v-if="!explorerStore.rootPath"
              class="px-4 py-2 rounded-[var(--radius-md)] border border-[var(--border-default)] text-[var(--text-sm)] text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] transition-colors"
              @click="explorerStore.chooseFolder()"
            >
              {{ t('exploration.open_folder') }}
            </button>
            <button
              class="px-4 py-2 rounded-[var(--radius-md)] border border-[var(--border-default)] text-[var(--text-sm)] text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] transition-colors"
              @click="fileInput?.click()"
            >
              {{ t('exploration.import_history') }}
            </button>
            <input
              ref="fileInput"
              type="file"
              accept=".md,.markdown,text/markdown"
              class="hidden"
              @change="handleImport"
            />
          </div>
          <p v-if="explorationStore.error" class="text-[var(--text-xs)] text-[var(--color-error)]">
            {{ explorationStore.error }}
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useExplorationStore } from '@/stores/exploration';
import { useExplorerStore } from '@/stores/explorer';
import { useSettingsStore } from '@/stores/settings';
import { useI18n } from '@/composables/useI18n';
import ExplorationChat from './ExplorationChat.vue';

const explorationStore = useExplorationStore();
const explorerStore = useExplorerStore();
const settingsStore = useSettingsStore();
const { t } = useI18n();
const fileInput = ref<HTMLInputElement | null>(null);
const baseline = ref(50);
let baselineDirty = false;

const steps = computed(() => [t('exploration.step_select'), t('exploration.step_graph'), t('exploration.step_learn')]);

const baselineLabel = computed(() => {
  if (baseline.value < 25) return t('exploration.baseline_zero');
  if (baseline.value < 50) return t('exploration.baseline_basic');
  if (baseline.value < 75) return t('exploration.baseline_solid');
  return t('exploration.baseline_advanced');
});

watch(
  () => settingsStore.settings.exploration?.default_baseline,
  (value) => {
    if (!baselineDirty && typeof value === 'number') {
      baseline.value = value;
    }
  },
  { immediate: true }
);

function onBaselineInput(event: Event) {
  baseline.value = Number((event.target as HTMLInputElement).value);
  baselineDirty = true;
}

async function persistBaseline() {
  baselineDirty = false;
  try {
    await settingsStore.queueUpdate((s) => {
      s.exploration.default_baseline = baseline.value;
    });
  } catch {
    // The settings store owns the user-visible error state.
  }
}

async function start() {
  await persistBaseline();
  await explorationStore.startSession(undefined, baseline.value);
}

async function handleImport(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  try {
    const markdown = await file.text();
    await explorationStore.importHistory(markdown, file.name);
  } finally {
    input.value = '';
  }
}
</script>
