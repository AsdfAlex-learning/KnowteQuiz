<template>
  <div class="p-4 space-y-6">
    <LLMConfigForm
      :llm="settings.llm"
      @update:llm="updateLLM"
    />

    <div class="border-t border-[var(--border-default)]" />

    <QuizDefaultsForm
      :model-value="settings.quiz"
      @update:model-value="updateQuiz"
    />

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
        {{ testing ? 'Testing...' : 'Test Connection' }}
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
        <span>{{ connectionResult.ok ? '\u2713 Connected' : '\u2717 ' + connectionResult.message }}</span>
      </div>

      <button
        class="w-full py-2 rounded-md text-sm font-medium bg-[var(--bg-elevated)] text-[var(--text-primary)] hover:bg-[var(--bg-active)] transition-colors"
        @click="handleSave"
      >
        Save Settings
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
        {{ backingUp ? 'Backing up...' : 'Backup Data Now' }}
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
        {{ restoring ? 'Restoring...' : 'Restore Latest Backup' }}
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
        {{ settingsStore.isCleaningUp ? 'Cleaning up...' : 'Clean Up Old Sessions' }}
      </button>

      <div
        v-if="settingsStore.cleanupResult"
        class="rounded-md border border-[var(--accent-green)]/30 bg-[var(--accent-green)]/10 p-3"
      >
        <p class="text-xs font-medium text-[var(--accent-green)]">
          Removed {{ settingsStore.cleanupResult.deleted_count }} old session(s)
        </p>
        <p class="mt-1 text-xs text-[var(--text-muted)]">
          {{ settingsStore.cleanupResult.remaining_count }} session(s) kept
        </p>
      </div>

      <div
        v-if="settingsStore.cleanupErr"
        class="text-xs text-[var(--color-error)]"
      >
        {{ settingsStore.cleanupErr }}
      </div>

      <div
        v-if="settingsStore.lastBackupResult"
        class="rounded-md border border-[var(--accent-green)]/30 bg-[var(--accent-green)]/10 p-3"
      >
        <p class="text-xs font-medium text-[var(--accent-green)]">
          Backed up {{ settingsStore.lastBackupResult.files.length }} files
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
          Restored {{ settingsStore.lastRestoreResult.files.length }} files
        </p>
        <p class="mt-1 truncate text-xs text-[var(--text-muted)]">
          {{ backupFolderName(settingsStore.lastRestoreResult.backup_dir) }}
        </p>
      </div>

      <div class="space-y-2 rounded-md border border-[var(--border-default)] bg-[var(--bg-elevated)] p-3">
        <div class="flex items-center justify-between gap-3">
          <div class="min-w-0">
            <p class="text-xs font-medium text-[var(--text-primary)]">Data Files</p>
            <p
              v-if="settingsStore.dataStatus"
              class="mt-1 truncate text-[11px] text-[var(--text-muted)]"
            >
              {{ settingsStore.dataStatus.data_dir }}
            </p>
          </div>
          <div class="flex items-center gap-1">
            <button
              class="shrink-0 rounded-md px-2 py-1 text-xs text-[var(--text-muted)] hover:bg-[var(--bg-active)]"
              title="Open data directory"
              @click="handleOpenDataDir"
            >
              Open Folder
            </button>
            <button
              class="shrink-0 rounded-md px-2 py-1 text-xs text-[var(--text-muted)] hover:bg-[var(--bg-active)]"
              @click="settingsStore.loadDataStatus()"
            >
              Refresh
            </button>
          </div>
        </div>

        <div
          v-if="settingsStore.dataStatus"
          class="space-y-1"
        >
          <div
            v-for="file in settingsStore.dataStatus.files"
            :key="file.name"
            class="grid grid-cols-[minmax(0,1fr)_auto] gap-x-2 gap-y-0.5 text-xs"
          >
            <span class="truncate text-[var(--text-primary)]">{{ file.name }}</span>
            <span :class="file.exists ? 'text-[var(--text-muted)]' : 'text-[var(--color-error)]'">
              {{ file.exists ? formatFileSize(file.size_bytes) : 'Missing' }}
            </span>
            <span
              v-if="file.exists && file.modified_at"
              class="col-span-2 truncate text-[11px] text-[var(--text-muted)]"
            >
              {{ formatModifiedAt(file.modified_at) }}
            </span>
          </div>
        </div>

        <p
          v-else-if="settingsStore.dataStatusError"
          class="text-xs text-[var(--color-error)]"
        >
          {{ settingsStore.dataStatusError }}
        </p>
        <p
          v-if="settingsStore.openDirErr"
          class="text-[11px] text-[var(--color-error)]"
        >
          {{ settingsStore.openDirErr }}
        </p>
      </div>

      <div
        v-if="settingsStore.error"
        class="rounded-md border border-[var(--color-error)]/40 bg-[var(--color-error)]/10 p-3"
      >
        <p class="text-xs font-medium text-[var(--color-error)]">Settings error</p>
        <p class="mt-1 text-xs leading-relaxed text-[var(--text-muted)]">
          {{ settingsStore.error }}
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useSettingsStore } from '@/stores/settings'
import LLMConfigForm from './LLMConfigForm.vue'
import QuizDefaultsForm from './QuizDefaultsForm.vue'
import type { ConnectionTestResult, SettingsLLM, SettingsQuiz } from '@/types/settings'

const settingsStore = useSettingsStore()
const settings = settingsStore.settings
const testing = ref(false)
const backingUp = ref(false)
const restoring = ref(false)
const connectionResult = ref<ConnectionTestResult | null>(null)

function updateLLM(llm: SettingsLLM) {
  settingsStore.settings.llm = llm
}

function updateQuiz(quiz: SettingsQuiz) {
  settingsStore.settings.quiz = quiz
}

async function handleTestConnection() {
  testing.value = true
  connectionResult.value = null
  try {
    connectionResult.value = await settingsStore.testConnection()
  } catch {
    connectionResult.value = {
      ok: false,
      kind: 'network',
      message: 'Connection failed',
      status: null,
    }
  } finally {
    testing.value = false
  }
}

async function handleSave() {
  await settingsStore.persistSettings()
}

async function handleBackup() {
  backingUp.value = true
  try {
    await settingsStore.backupDataNow()
  } catch {
    // The store owns the user-visible error state.
  } finally {
    backingUp.value = false
  }
}

async function handleRestore() {
  restoring.value = true
  try {
    await settingsStore.restoreLatestBackupNow()
  } catch {
    // The store owns the user-visible error state.
  } finally {
    restoring.value = false
  }
}

async function handleOpenDataDir() {
  await settingsStore.openDataDirNow()
}

async function handleCleanupSessions() {
  await settingsStore.cleanupSessionsNow()
}

function backupFolderName(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).pop() || path
}

function formatFileSize(sizeBytes: number): string {
  if (sizeBytes < 1024) return `${sizeBytes} B`
  return `${Math.round(sizeBytes / 1024)} KB`
}

function formatModifiedAt(value: string): string {
  return value.slice(0, 16).replace('T', ' ')
}

onMounted(() => {
  settingsStore.loadSettings()
  settingsStore.loadDataStatus()
})
</script>
