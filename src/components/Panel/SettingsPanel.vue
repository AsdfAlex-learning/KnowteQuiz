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

onMounted(() => {
  settingsStore.loadSettings()
})
</script>
