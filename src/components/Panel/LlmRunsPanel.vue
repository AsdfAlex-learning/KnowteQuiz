<template>
  <div class="h-full flex flex-col">
    <!-- Toolbar -->
    <div class="px-3 py-2 border-b border-[var(--border-default)] flex items-center gap-2">
      <span class="text-[11px] font-medium text-[var(--text-muted)] uppercase tracking-wider">
        {{ t('llm_runs.title') }}
      </span>
      <span class="text-[11px] text-[var(--text-faint)]">
        {{ t('llm_runs.items_count', { count: runs.length }) }}
      </span>
      <button
        class="ml-auto px-2 py-0.5 text-[10px] rounded text-[var(--text-muted)] hover:bg-[var(--bg-elevated)] hover:text-[var(--text-secondary)] transition-colors disabled:opacity-40"
        :disabled="loading"
        :title="t('llm_runs.refresh')"
        @click="loadRuns"
      >
        {{ t('llm_runs.refresh') }}
      </button>
    </div>

    <div v-if="error" class="px-3 py-2 text-xs text-[var(--color-error)] border-b border-[var(--border-default)]">
      {{ error }}
    </div>

    <!-- List -->
    <div class="flex-1 overflow-y-auto">
      <div v-if="loading && runs.length === 0" class="p-6 text-center text-xs text-[var(--text-faint)]">
        {{ t('common.loading') }}
      </div>
      <div v-else-if="runs.length === 0" class="p-6 text-center">
        <p class="text-sm text-[var(--text-muted)]">{{ t('llm_runs.empty') }}</p>
        <p class="mt-1 text-xs text-[var(--text-faint)]">{{ t('llm_runs.empty_hint') }}</p>
      </div>
      <div v-else>
        <div
          v-for="run in runs"
          :key="run.id"
          class="border-b border-[var(--border-subtle)] cursor-pointer hover:bg-[var(--bg-elevated)]/50 transition-colors"
          @click="toggleExpand(run.id)"
        >
          <!-- Row -->
          <div class="px-3 py-2 flex items-center gap-2">
            <span class="shrink-0 px-1.5 py-0.5 text-[10px] rounded" :class="kindBadgeClass(run.kind)">
              {{ kindLabel(run.kind) }}
            </span>
            <span class="text-[11px] text-[var(--text-faint)] font-mono">
              {{ formatTime(run.ts_start) }}
            </span>
            <span class="ml-auto flex items-center gap-2 text-[11px] font-mono">
              <span class="text-[var(--text-muted)]">{{ formatDuration(run.duration_ms) }}</span>
              <span v-if="run.usage" class="text-[var(--text-faint)]" :title="usageTitle(run)">
                {{ run.usage.total_tokens ?? '—' }}t
              </span>
              <span v-else class="text-[var(--text-faint)]" :title="t('llm_runs.usage_unavailable')">—</span>
              <span
                class="w-1.5 h-1.5 rounded-full"
                :class="run.error ? 'bg-[var(--color-error)]' : 'bg-[var(--color-success)]'"
              />
            </span>
          </div>

          <!-- Expanded detail -->
          <div v-if="expandedId === run.id" class="px-3 pb-3 space-y-2">
            <div v-if="expandedError" class="text-[11px] text-[var(--color-error)]">
              {{ expandedError }}
            </div>
            <div v-else-if="contentLoading" class="text-[11px] text-[var(--text-faint)]">
              {{ t('common.loading') }}
            </div>
            <template v-else-if="expandedRecord">
              <!-- Request params -->
              <div class="rounded bg-[var(--bg-base)] p-2">
                <div class="flex items-center justify-between mb-1">
                  <span class="text-[10px] font-semibold uppercase tracking-wider text-[var(--text-faint)]">
                    {{ t('llm_runs.request_params') }}
                  </span>
                  <span class="text-[10px] text-[var(--text-faint)] font-mono">
                    {{ expandedRecord.endpoint_host }}
                  </span>
                </div>
                <pre class="text-[11px] font-mono text-[var(--text-secondary)] whitespace-pre-wrap break-all">{{
                  paramsText
                }}</pre>
              </div>

              <!-- Prompt -->
              <div class="rounded bg-[var(--bg-base)] p-2">
                <div class="flex items-center justify-between mb-1">
                  <span class="text-[10px] font-semibold uppercase tracking-wider text-[var(--text-faint)]">
                    {{ t('llm_runs.prompt') }}
                  </span>
                  <button
                    class="px-1.5 py-0.5 text-[10px] rounded text-[var(--text-muted)] hover:bg-[var(--bg-elevated)] transition-colors"
                    @click.stop="copy(promptText, 'prompt')"
                  >
                    {{ copied === 'prompt' ? t('llm_runs.copied') : t('llm_runs.copy') }}
                  </button>
                </div>
                <pre
                  class="text-[11px] font-mono text-[var(--text-secondary)] whitespace-pre-wrap break-all max-h-48 overflow-y-auto"
                  >{{ promptText || t('llm_runs.not_recorded') }}</pre>
              </div>

              <!-- Raw response -->
              <div class="rounded bg-[var(--bg-base)] p-2">
                <div class="flex items-center justify-between mb-1">
                  <span class="text-[10px] font-semibold uppercase tracking-wider text-[var(--text-faint)]">
                    {{ t('llm_runs.response') }}
                  </span>
                  <button
                    class="px-1.5 py-0.5 text-[10px] rounded text-[var(--text-muted)] hover:bg-[var(--bg-elevated)] transition-colors"
                    @click.stop="copy(responseText, 'response')"
                  >
                    {{ copied === 'response' ? t('llm_runs.copied') : t('llm_runs.copy') }}
                  </button>
                </div>
                <pre
                  class="text-[11px] font-mono text-[var(--text-secondary)] whitespace-pre-wrap break-all max-h-48 overflow-y-auto"
                  >{{ responseText || t('llm_runs.not_recorded') }}</pre>
              </div>
            </template>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { getLlmRun, getRunContent, listLlmRuns } from '@/services/llmRuns';
import type { LlmRunKind, LlmRunRecord, LlmRunSummary } from '@/types/llmRun';
import { useI18n } from '@/composables/useI18n';

const { t } = useI18n();

const runs = ref<LlmRunSummary[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);

const expandedId = ref<string | null>(null);
const expandedRecord = ref<LlmRunRecord | null>(null);
const expandedError = ref<string | null>(null);
const contentLoading = ref(false);
const promptText = ref('');
const responseText = ref('');
const copied = ref<'prompt' | 'response' | null>(null);

const paramsText = computed(() => {
  const params = expandedRecord.value?.request_params;
  if (params === undefined || params === null) return t('llm_runs.not_recorded');
  try {
    return JSON.stringify(params, null, 2);
  } catch {
    return String(params);
  }
});

async function loadRuns() {
  loading.value = true;
  error.value = null;
  try {
    runs.value = await listLlmRuns({ limit: 100 });
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

async function toggleExpand(id: string) {
  if (expandedId.value === id) {
    expandedId.value = null;
    expandedRecord.value = null;
    return;
  }
  expandedId.value = id;
  expandedRecord.value = null;
  expandedError.value = null;
  promptText.value = '';
  responseText.value = '';
  copied.value = null;
  contentLoading.value = true;
  try {
    const record = await getLlmRun(id);
    expandedRecord.value = record;
    const [prompt, response] = await Promise.all([getRunContent(id, 'prompt'), getRunContent(id, 'response')]);
    promptText.value = prompt;
    responseText.value = response;
  } catch (e) {
    expandedError.value = String(e);
  } finally {
    contentLoading.value = false;
  }
}

async function copy(text: string, which: 'prompt' | 'response') {
  if (!text) return;
  try {
    await navigator.clipboard.writeText(text);
    copied.value = which;
    setTimeout(() => {
      if (copied.value === which) copied.value = null;
    }, 1500);
  } catch {
    // Clipboard unavailable (e.g. insecure context) — silently ignore.
  }
}

function kindBadgeClass(kind: LlmRunKind): string {
  switch (kind) {
    case 'quiz':
      return 'bg-[var(--accent-purple)]/20 text-[var(--accent-purple)]';
    case 'quiz_fixup':
      return 'bg-[var(--accent-yellow)]/20 text-[var(--accent-yellow)]';
    case 'diagnosis_initial':
    case 'diagnosis_followup':
    case 'diagnosis_report':
      return 'bg-[var(--accent-blue)]/20 text-[var(--accent-blue)]';
    default:
      return 'bg-[var(--bg-active)] text-[var(--text-muted)]';
  }
}

function kindLabel(kind: LlmRunKind): string {
  return t(`llm_runs.kind_${kind}`);
}

function formatTime(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;
  return date.toLocaleTimeString();
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

function usageTitle(run: LlmRunSummary): string {
  const u = run.usage;
  if (!u) return '';
  return `prompt ${u.prompt_tokens ?? '—'} / completion ${u.completion_tokens ?? '—'} / total ${u.total_tokens ?? '—'} (${run.usage_source})`;
}

onMounted(loadRuns);
</script>
