<template>
  <div class="p-4 space-y-6">
    <LLMConfigForm
      :llm="settings.llm"
      @update:llm="updateLLM"
    />

    <div class="border-t border-[#45475a]" />

    <QuizDefaultsForm
      :model-value="settings.quiz"
      @update:model-value="updateQuiz"
    />

    <div class="border-t border-[#45475a]" />

    <div class="space-y-3">
      <button
        class="w-full py-2 rounded-md text-sm font-medium transition-colors"
        :class="
          testing
            ? 'bg-[#45475a] text-[#a6adc8] cursor-wait'
            : 'bg-[#cba6f7] text-[#1e1e2e] hover:bg-[#b4befe]'
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
          connectionResult
            ? 'bg-[#a6e3a1]/10 text-[#a6e3a1]'
            : 'bg-[#f38ba8]/10 text-[#f38ba8]'
        "
      >
        <span>{{ connectionResult ? '✓ Connected' : '✗ Connection failed' }}</span>
      </div>

      <button
        class="w-full py-2 rounded-md text-sm font-medium bg-[#313244] text-[#f8f8f2] hover:bg-[#45475a] transition-colors"
        @click="handleSave"
      >
        Save Settings
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useSettingsStore } from '@/stores/settings'
import { testConnection } from '@/services/settings'
import LLMConfigForm from './LLMConfigForm.vue'
import QuizDefaultsForm from './QuizDefaultsForm.vue'
import type { SettingsLLM, SettingsQuiz } from '@/types/settings'

const settingsStore = useSettingsStore()
const settings = settingsStore.settings
const testing = ref(false)
const connectionResult = ref<boolean | null>(null)

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
    const ok = await testConnection()
    connectionResult.value = ok
  } catch {
    connectionResult.value = false
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
