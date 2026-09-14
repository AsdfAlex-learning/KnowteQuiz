<template>
  <div class="p-4 space-y-6">
    <!-- Language selector -->
    <div class="space-y-2">
      <label class="text-xs font-medium text-[var(--text-secondary)]">{{ t('settings_page.language') }}</label>
      <select
        :value="settings.ui_language"
        class="w-full px-2 py-1.5 text-sm bg-[var(--bg-base)] text-[var(--text-primary)] border border-[var(--border-default)] rounded focus:border-[var(--border-focus)] focus:outline-none"
        @change="handleLanguageChange"
      >
        <option v-for="loc in availableLocales" :key="loc.value" :value="loc.value">
          {{ loc.label }}
        </option>
      </select>
    </div>

    <div class="border-t border-[var(--border-default)]" />

    <LLMConfigForm :llm="settings.llm" @update:llm="updateLLM" />

    <div class="border-t border-[var(--border-default)]" />

    <QuizDefaultsForm :model-value="settings.quiz" @update:model-value="updateQuiz" />

    <div class="border-t border-[var(--border-default)]" />

    <div class="space-y-3">
      <button
        class="w-full py-2 rounded-md text-sm font-medium transition-colors btn-press"
        :class="
          testing
            ? 'bg-[var(--bg-active)] text-[var(--text-muted)] cursor-wait'
            : 'bg-[var(--accent-purple)] text-[var(--bg-base)] hover:bg-[var(--accent-lavender)]'
        "
        :disabled="testing"
        @click="handleTestConnection"
      >
        {{ testing ? t('settings_page.testing') : t('settings_page.test_connection') }}
      </button>

      <div
        v-if="connectionResult !== null"
        class="flex items-center gap-2 text-xs px-3 py-2 rounded-md"
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

      <button
        class="w-full py-2 rounded-md text-sm font-medium bg-[var(--bg-elevated)] text-[var(--text-primary)] hover:bg-[var(--bg-active)] transition-colors disabled:opacity-50"
        :disabled="probing"
        @click="handleProbeLlm"
      >
        {{ probing ? t('settings_page.probing') : t('settings_page.probe_llm') }}
      </button>

      <div
        v-if="settingsStore.llmCapabilities"
        class="rounded-md border border-[var(--border-default)] bg-[var(--bg-base)] p-3 space-y-1"
      >
        <p class="text-xs font-medium text-[var(--text-primary)]">
          {{ t('settings_page.model') }}: {{ settingsStore.llmCapabilities.default_model }}
        </p>
        <p class="text-xs text-[var(--text-muted)]">
          {{ t('settings_page.streaming') }}:
          {{
            settingsStore.llmCapabilities.supports_streaming ? '✓ ' + t('common.confirm') : '✗ ' + t('common.cancel')
          }}
        </p>
        <p class="text-xs text-[var(--text-muted)]">
          {{ t('settings_page.json_object') }}:
          {{
            settingsStore.llmCapabilities.supports_response_format
              ? '✓ ' + t('common.confirm')
              : '✗ ' + t('common.cancel')
          }}
        </p>
        <p
          v-if="settingsStore.llmCapabilities.available_models.length > 0"
          class="text-xs text-[var(--text-muted)] mt-1"
        >
          {{ t('settings_page.available_models') }}: {{ settingsStore.llmCapabilities.available_models.join(', ') }}
        </p>
      </div>
      <div v-if="settingsStore.probeErr" class="text-xs text-[var(--color-error)]">
        {{ settingsStore.probeErr }}
      </div>

      <button
        class="w-full py-2 rounded-md text-sm font-medium transition-colors btn-press"
        :class="
          saving
            ? 'bg-[var(--bg-active)] text-[var(--text-muted)] cursor-wait'
            : 'bg-[var(--bg-elevated)] text-[var(--text-primary)] hover:bg-[var(--bg-active)]'
        "
        :disabled="saving"
        @click="handleSave"
      >
        {{ saving ? t('settings_page.saving') : t('settings_page.save') }}
      </button>

      <button
        class="w-full py-2 rounded-md text-sm font-medium transition-colors btn-press"
        :class="
          backingUp
            ? 'bg-[var(--bg-active)] text-[var(--text-muted)] cursor-wait'
            : 'bg-[var(--bg-elevated)] text-[var(--text-primary)] hover:bg-[var(--bg-active)]'
        "
        :disabled="backingUp"
        @click="handleBackup"
      >
        {{ backingUp ? t('settings_page.backing_up') : t('settings_page.backup') }}
      </button>

      <button
        class="w-full py-2 rounded-md text-sm font-medium transition-colors btn-press"
        :class="
          restoring
            ? 'bg-[var(--bg-active)] text-[var(--text-muted)] cursor-wait'
            : 'bg-[var(--bg-elevated)] text-[var(--text-primary)] hover:bg-[var(--bg-active)]'
        "
        :disabled="restoring"
        @click="handleRestore"
      >
        {{ restoring ? t('settings_page.restoring') : t('settings_page.restore') }}
      </button>

      <button
        class="w-full py-2 rounded-md text-sm font-medium transition-colors btn-press"
        :class="
          settingsStore.isCleaningUp
            ? 'bg-[var(--bg-active)] text-[var(--text-muted)] cursor-wait'
            : 'bg-[var(--bg-elevated)] text-[var(--text-primary)] hover:bg-[var(--bg-active)]'
        "
        :disabled="settingsStore.isCleaningUp"
        @click="handleCleanupSessions"
      >
        {{ settingsStore.isCleaningUp ? t('settings_page.cleaning_up') : t('settings_page.cleanup_sessions') }}
      </button>

      <div
        v-if="settingsStore.cleanupResult"
        class="rounded-md border border-[var(--accent-green)]/30 bg-[var(--accent-green)]/10 p-3"
      >
        <p class="text-xs font-medium text-[var(--accent-green)]">
          {{ t('settings_page.removed_sessions', { count: settingsStore.cleanupResult.deleted_count }) }}
        </p>
        <p class="mt-1 text-xs text-[var(--text-muted)]">
          {{ t('settings_page.kept_sessions', { count: settingsStore.cleanupResult.remaining_count }) }}
        </p>
      </div>

      <div v-if="settingsStore.cleanupErr" class="text-xs text-[var(--color-error)]">
        {{ settingsStore.cleanupErr }}
      </div>

      <div
        v-if="settingsStore.lastBackupResult"
        class="rounded-md border border-[var(--accent-green)]/30 bg-[var(--accent-green)]/10 p-3"
      >
        <p class="text-xs font-medium text-[var(--accent-green)]">
          {{ t('settings_page.backed_up_files', { count: settingsStore.lastBackupResult.files.length }) }}
        </p>
        <p class="mt-1 truncate text-xs text-[var(--text-muted)]">
          {{ backupFolderName(settingsStore.lastBackupResult.backup_dir) }}
        </p>
      </div>

      <div
        v-if="settingsStore.lastRestoreResult"
        class="rounded-md border border-[var(--accent-green)]/30 bg-[var(--accent-green)]/10 p-3"
      >
        <p class="text-xs font-medium text-[var(--accent-green)]">
          {{ t('settings_page.restored_files', { count: settingsStore.lastRestoreResult.files.length }) }}
        </p>
        <p class="mt-1 truncate text-xs text-[var(--text-muted)]">
          {{ backupFolderName(settingsStore.lastRestoreResult.backup_dir) }}
        </p>
      </div>

      <div class="space-y-2 rounded-md border border-[var(--border-default)] bg-[var(--bg-elevated)] p-3">
        <div class="flex items-center justify-between gap-3">
          <div class="min-w-0">
            <p class="text-xs font-medium text-[var(--text-primary)]">{{ t('settings_page.data_files') }}</p>
            <p v-if="settingsStore.dataStatus" class="mt-1 truncate text-[11px] text-[var(--text-muted)]">
              {{ settingsStore.dataStatus.data_dir }}
            </p>
          </div>
          <div class="flex items-center gap-1">
            <button
              class="shrink-0 rounded-md px-2 py-1 text-xs text-[var(--text-muted)] hover:bg-[var(--bg-active)]"
              :title="t('settings_page.open_data_dir')"
              @click="handleOpenDataDir"
            >
              {{ t('settings_page.open_data_dir') }}
            </button>
            <button
              class="shrink-0 rounded-md px-2 py-1 text-xs text-[var(--text-muted)] hover:bg-[var(--bg-active)]"
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
              {{ file.exists ? formatFileSize(file.size_bytes) : t('common.missing') }}
            </span>
            <span
              v-if="file.exists && file.modified_at"
              class="col-span-2 truncate text-[11px] text-[var(--text-muted)]"
            >
              {{ formatModifiedAt(file.modified_at) }}
            </span>
          </div>
        </div>

        <p v-else-if="settingsStore.dataStatusError" class="text-xs text-[var(--color-error)]">
          {{ settingsStore.dataStatusError }}
        </p>
        <p v-if="settingsStore.openDirErr" class="text-[11px] text-[var(--color-error)]">
          {{ settingsStore.openDirErr }}
        </p>
      </div>

      <div
        v-if="settingsStore.error"
        class="rounded-md border border-[var(--color-error)]/40 bg-[var(--color-error)]/10 p-3"
      >
        <p class="text-xs font-medium text-[var(--color-error)]">{{ t('settings_page.settings_error') }}</p>
        <p class="mt-1 text-xs leading-relaxed text-[var(--text-muted)]">
          {{ settingsStore.error }}
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useSettingsStore } from '@/stores/settings';
import { useI18n } from '@/composables/useI18n';
import LLMConfigForm from './LLMConfigForm.vue';
import QuizDefaultsForm from './QuizDefaultsForm.vue';
import type { ConnectionTestResult, SettingsLLM, SettingsQuiz } from '@/types/settings';

const settingsStore = useSettingsStore();
const settings = settingsStore.settings;
const { t, availableLocales, setLocale } = useI18n();
const testing = ref(false);
const backingUp = ref(false);
const restoring = ref(false);
const probing = ref(false);
const saving = ref(false);
const connectionResult = ref<ConnectionTestResult | null>(null);

function updateLLM(llm: SettingsLLM) {
  settingsStore.settings.llm = llm;
}

function updateQuiz(quiz: SettingsQuiz) {
  settingsStore.settings.quiz = quiz;
}

function handleLanguageChange(e: Event) {
  const target = e.target as HTMLSelectElement;
  setLocale(target.value);
}

async function handleTestConnection() {
  testing.value = true;
  connectionResult.value = null;
  try {
    connectionResult.value = await settingsStore.testConnection();
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

async function handleSave() {
  saving.value = true;
  try {
    await settingsStore.persistSettings();
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
  if (!window.confirm(t('settings_page.confirm_restore'))) {
    return;
  }
  restoring.value = true;
  try {
    await settingsStore.restoreLatestBackupNow();
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

function backupFolderName(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).pop() || path;
}

function formatFileSize(sizeBytes: number): string {
  if (sizeBytes < 1024) return `${sizeBytes} B`;
  return `${Math.round(sizeBytes / 1024)} KB`;
}

function formatModifiedAt(value: string): string {
  return value.slice(0, 16).replace('T', ' ');
}

onMounted(() => {
  settingsStore.loadSettings();
  settingsStore.loadDataStatus();
});
</script>
