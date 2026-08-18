<template>
  <Teleport to="body">
    <Transition name="modal">
      <div
        v-if="modelValue"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
        @mousedown.self="$emit('update:modelValue', false)"
      >
        <div
          class="relative flex flex-col w-full max-w-2xl max-h-[85vh] bg-[var(--bg-base)] rounded-xl border border-[var(--border-default)] shadow-2xl"
        >
          <!-- Header -->
          <div class="flex items-center justify-between px-6 py-4 border-b border-[var(--border-default)]">
            <h2 class="text-lg font-semibold text-[var(--text-primary)]">{{ t('settings_page.title') }}</h2>
            <button
              class="w-8 h-8 flex items-center justify-center rounded-lg text-[var(--text-muted)] hover:bg-[var(--bg-active)] hover:text-[var(--text-primary)] transition-colors"
              @click="$emit('update:modelValue', false)"
            >
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M4 4l8 8M12 4l-8 8" />
              </svg>
            </button>
          </div>

          <!-- Body (scrollable) -->
          <div class="flex-1 overflow-y-auto px-6 py-4 space-y-6">
            <!-- Language settings -->
            <div class="space-y-3">
              <h3 class="text-sm font-medium text-[var(--text-primary)]">{{ t('settings_page.language') }}</h3>
              <div class="grid grid-cols-2 gap-3">
                <div class="space-y-1">
                  <label class="text-xs text-[var(--text-muted)]">{{ t('settings_page.ui_language') }}</label>
                  <select
                    :value="localSettings.ui_language"
                    class="w-full px-3 py-2 text-sm bg-[var(--bg-elevated)] text-[var(--text-primary)] border border-[var(--border-default)] rounded-lg focus:border-[var(--border-focus)] focus:outline-none"
                    @change="handleLanguageChange"
                  >
                    <option v-for="loc in availableLocales" :key="loc.value" :value="loc.value">
                      {{ loc.label }}
                    </option>
                  </select>
                </div>
                <div class="space-y-1">
                  <label class="text-xs text-[var(--text-muted)]">{{ t('settings_page.quiz_language') }}</label>
                  <select
                    :value="localSettings.quiz.default_language"
                    class="w-full px-3 py-2 text-sm bg-[var(--bg-elevated)] text-[var(--text-primary)] border border-[var(--border-default)] rounded-lg focus:border-[var(--border-focus)] focus:outline-none"
                    @change="handleQuizLanguageChange"
                  >
                    <option value="zh">中文</option>
                    <option value="en">English</option>
                    <option value="ja">日本語</option>
                    <option value="ko">한국어</option>
                  </select>
                </div>
              </div>
            </div>

            <div class="border-t border-[var(--border-default)]" />

            <!-- LLM Config -->
            <LLMConfigForm :llm="localSettings.llm" @update:llm="updateLLM" />

            <div class="border-t border-[var(--border-default)]" />

            <!-- Quiz Defaults -->
            <QuizDefaultsForm :model-value="localSettings.quiz" @update:model-value="updateQuiz" />

            <div class="border-t border-[var(--border-default)]" />

            <!-- Data Management -->
            <div class="space-y-3">
              <h3 class="text-sm font-medium text-[var(--text-primary)]">{{ t('settings_page.data_files') }}</h3>

              <div class="flex gap-2">
                <button
                  class="flex-1 py-2 rounded-lg text-sm font-medium transition-colors btn-press"
                  :class="
                    testing
                      ? 'bg-[var(--bg-active)] text-[var(--text-muted)] cursor-wait'
                      : 'bg-[var(--accent-purple)] text-[var(--bg-base)] hover:bg-[var(--accent-lavender)]'
                  "
                  :disabled="testing"
                  @click="handleTestConnection"
                >
                  {{ testing ? '...' : t('settings_page.test_connection') }}
                </button>

                <button
                  class="flex-1 py-2 rounded-lg text-sm font-medium bg-[var(--bg-elevated)] text-[var(--text-primary)] hover:bg-[var(--bg-active)] transition-colors disabled:opacity-50"
                  :disabled="probing"
                  @click="handleProbeLlm"
                >
                  {{ probing ? '...' : t('settings_page.probe_llm') }}
                </button>
              </div>

              <div
                v-if="connectionResult !== null"
                class="flex items-center gap-2 text-xs px-3 py-2 rounded-lg"
                :class="
                  connectionResult.ok
                    ? 'bg-[var(--accent-green)]/10 text-[var(--accent-green)]'
                    : 'bg-[var(--color-error)]/10 text-[var(--color-error)]'
                "
              >
                <span>{{
                  connectionResult.ok ? '\u2713 ' + t('settings_page.connected') : '\u2717 ' + connectionResult.message
                }}</span>
              </div>

              <div
                v-if="settingsStore.llmCapabilities"
                class="rounded-lg border border-[var(--border-default)] bg-[var(--bg-elevated)] p-3 space-y-1"
              >
                <p class="text-xs font-medium text-[var(--text-primary)]">
                  Model: {{ settingsStore.llmCapabilities.default_model }}
                </p>
                <p class="text-xs text-[var(--text-muted)]">
                  Streaming: {{ settingsStore.llmCapabilities.supports_streaming ? '✓' : '✗' }}
                </p>
                <p class="text-xs text-[var(--text-muted)]">
                  json_object: {{ settingsStore.llmCapabilities.supports_response_format ? '✓' : '✗' }}
                </p>
                <p
                  v-if="settingsStore.llmCapabilities.available_models.length > 0"
                  class="text-xs text-[var(--text-muted)] mt-1"
                >
                  {{ settingsStore.llmCapabilities.available_models.join(', ') }}
                </p>
              </div>
              <div v-if="settingsStore.probeErr" class="text-xs text-[var(--color-error)]">
                {{ settingsStore.probeErr }}
              </div>

              <div class="flex gap-2">
                <button
                  class="flex-1 py-2 rounded-lg text-sm font-medium bg-[var(--bg-elevated)] text-[var(--text-primary)] hover:bg-[var(--bg-active)] transition-colors btn-press"
                  :disabled="backingUp"
                  @click="handleBackup"
                >
                  {{ backingUp ? '...' : t('settings_page.backup') }}
                </button>
                <button
                  class="flex-1 py-2 rounded-lg text-sm font-medium bg-[var(--bg-elevated)] text-[var(--text-primary)] hover:bg-[var(--bg-active)] transition-colors btn-press"
                  :disabled="restoring"
                  @click="handleRestore"
                >
                  {{ restoring ? '...' : t('settings_page.restore') }}
                </button>
              </div>

              <button
                class="w-full py-2 rounded-lg text-sm font-medium bg-[var(--bg-elevated)] text-[var(--text-primary)] hover:bg-[var(--bg-active)] transition-colors btn-press"
                :disabled="settingsStore.isCleaningUp"
                @click="handleCleanupSessions"
              >
                {{ settingsStore.isCleaningUp ? '...' : t('settings_page.cleanup_sessions') }}
              </button>

              <div
                v-if="settingsStore.cleanupResult"
                class="rounded-lg border border-[var(--accent-green)]/30 bg-[var(--accent-green)]/10 p-3"
              >
                <p class="text-xs font-medium text-[var(--accent-green)]">
                  Removed {{ settingsStore.cleanupResult.deleted_count }} session(s)
                </p>
              </div>

              <!-- Data Files -->
              <div class="space-y-2 rounded-lg border border-[var(--border-default)] bg-[var(--bg-elevated)] p-3">
                <div class="flex items-center justify-between gap-3">
                  <div class="min-w-0">
                    <p v-if="settingsStore.dataStatus" class="mt-1 truncate text-[11px] text-[var(--text-muted)]">
                      {{ settingsStore.dataStatus.data_dir }}
                    </p>
                  </div>
                  <div class="flex items-center gap-1">
                    <button
                      class="shrink-0 rounded-lg px-2 py-1 text-xs text-[var(--text-muted)] hover:bg-[var(--bg-active)]"
                      @click="handleOpenDataDir"
                    >
                      {{ t('settings_page.open_data_dir') }}
                    </button>
                    <button
                      class="shrink-0 rounded-lg px-2 py-1 text-xs text-[var(--text-muted)] hover:bg-[var(--bg-active)]"
                      @click="settingsStore.loadDataStatus()"
                    >
                      {{ t('settings_page.refresh') }}
                    </button>
                  </div>
                </div>

                <div v-if="settingsStore.dataStatus" class="space-y-1">
                  <div
                    v-for="file in settingsStore.dataStatus.files"
                    :key="file.name"
                    class="grid grid-cols-[minmax(0,1fr)_auto] gap-x-2 gap-y-0.5 text-xs"
                  >
                    <span class="truncate text-[var(--text-primary)]">{{ file.name }}</span>
                    <span :class="file.exists ? 'text-[var(--text-muted)]' : 'text-[var(--color-error)]'">
                      {{ file.exists ? formatFileSize(file.size_bytes) : 'Missing' }}
                    </span>
                  </div>
                </div>
              </div>

              <div
                v-if="settingsStore.error"
                class="rounded-lg border border-[var(--color-error)]/40 bg-[var(--color-error)]/10 p-3"
              >
                <p class="text-xs font-medium text-[var(--color-error)]">Error</p>
                <p class="mt-1 text-xs leading-relaxed text-[var(--text-muted)]">
                  {{ settingsStore.error }}
                </p>
              </div>
            </div>
          </div>

          <!-- Footer -->
          <div class="flex items-center justify-end gap-3 px-6 py-4 border-t border-[var(--border-default)]">
            <button
              class="px-5 py-2.5 rounded-lg text-sm font-medium bg-[var(--bg-elevated)] text-[var(--text-primary)] hover:bg-[var(--bg-active)] transition-colors"
              @click="$emit('update:modelValue', false)"
            >
              {{ t('common.cancel') }}
            </button>
            <button
              class="px-5 py-2.5 rounded-lg text-sm font-medium transition-colors btn-press"
              :class="
                saving
                  ? 'bg-[var(--bg-active)] text-[var(--text-muted)] cursor-wait'
                  : 'bg-[var(--accent-purple)] text-[var(--bg-base)] hover:bg-[var(--accent-lavender)]'
              "
              :disabled="saving"
              @click="handleSaveAndQuit"
            >
              {{ saving ? '...' : t('settings_page.save') }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, reactive, watch, onMounted } from 'vue';
import { useSettingsStore } from '@/stores/settings';
import { useI18n } from '@/composables/useI18n';
import LLMConfigForm from './LLMConfigForm.vue';
import QuizDefaultsForm from './QuizDefaultsForm.vue';
import type { ConnectionTestResult, Settings, SettingsLLM, SettingsQuiz } from '@/types/settings';

const props = defineProps<{
  modelValue: boolean;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
}>();

const settingsStore = useSettingsStore();
const { t, availableLocales, setLocale } = useI18n();

// Local copy for buffering changes
const localSettings = reactive<Settings>(JSON.parse(JSON.stringify(settingsStore.settings)));

// Sync when modal opens
watch(
  () => props.modelValue,
  (open) => {
    if (open) {
      Object.assign(localSettings, JSON.parse(JSON.stringify(settingsStore.settings)));
    }
  }
);

const testing = ref(false);
const backingUp = ref(false);
const restoring = ref(false);
const probing = ref(false);
const saving = ref(false);
const connectionResult = ref<ConnectionTestResult | null>(null);

function updateLLM(llm: SettingsLLM) {
  localSettings.llm = llm;
}

function updateQuiz(quiz: SettingsQuiz) {
  localSettings.quiz = quiz;
}

function handleLanguageChange(e: Event) {
  const target = e.target as HTMLSelectElement;
  localSettings.ui_language = target.value;
  setLocale(target.value);
}

function handleQuizLanguageChange(e: Event) {
  const target = e.target as HTMLSelectElement;
  localSettings.quiz.default_language = target.value;
}

async function handleTestConnection() {
  testing.value = true;
  connectionResult.value = null;
  try {
    // Test with current LLM config from local settings
    const original = settingsStore.settings.llm;
    settingsStore.settings.llm = localSettings.llm;
    connectionResult.value = await settingsStore.testConnection();
    settingsStore.settings.llm = original;
  } catch {
    connectionResult.value = {
      ok: false,
      kind: 'network',
      message: 'Connection failed',
      status: null,
    };
  } finally {
    testing.value = false;
  }
}

async function handleSaveAndQuit() {
  saving.value = true;
  try {
    // Apply local settings to store
    Object.assign(settingsStore.settings, JSON.parse(JSON.stringify(localSettings)));
    await settingsStore.persistSettings();
    emit('update:modelValue', false);
  } catch {
    // The store owns the user-visible error state.
  } finally {
    saving.value = false;
  }
}

async function handleBackup() {
  backingUp.value = true;
  try {
    await settingsStore.backupDataNow();
  } catch {
    // The store owns the user-visible error state.
  } finally {
    backingUp.value = false;
  }
}

async function handleRestore() {
  if (
    !window.confirm(
      'Restore will overwrite current settings and mistakes from the latest backup. A pre-restore snapshot is taken automatically. Continue?'
    )
  ) {
    return;
  }
  restoring.value = true;
  try {
    await settingsStore.restoreLatestBackupNow();
    // Reload local settings after restore
    Object.assign(localSettings, JSON.parse(JSON.stringify(settingsStore.settings)));
  } catch {
    // The store owns the user-visible error state.
  } finally {
    restoring.value = false;
  }
}

async function handleOpenDataDir() {
  await settingsStore.openDataDirNow();
}

async function handleCleanupSessions() {
  await settingsStore.cleanupSessionsNow();
}

async function handleProbeLlm() {
  probing.value = true;
  try {
    await settingsStore.probeLlmNow();
  } finally {
    probing.value = false;
  }
}

function formatFileSize(sizeBytes: number): string {
  if (sizeBytes < 1024) return `${sizeBytes} B`;
  return `${Math.round(sizeBytes / 1024)} KB`;
}

onMounted(() => {
  settingsStore.loadDataStatus();
});
</script>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}
.modal-enter-active > div,
.modal-leave-active > div {
  transition:
    transform 0.2s ease,
    opacity 0.2s ease;
}
.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
.modal-enter-from > div,
.modal-leave-to > div {
  transform: scale(0.95);
  opacity: 0;
}
</style>
