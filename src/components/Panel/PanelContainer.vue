<template>
  <div class="h-full flex flex-col bg-[var(--bg-base)]">
    <PanelTabs v-model="activeTab" />
    <div class="flex-1 overflow-y-auto">
      <div v-if="activeTab === 'quiz'" class="h-full flex flex-col">
        <QuizGenerator v-if="!quizStore.hasSession && !quizStore.isGenerating" />
        <QuizSession v-else :mode="quizStore.mode" />
      </div>
      <ErrorBook v-else-if="activeTab === 'mistakes'" />
      <SettingsPanel v-else-if="activeTab === 'settings'" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import PanelTabs from './PanelTabs.vue'
import type { PanelTab } from './PanelTabs.vue'
import QuizGenerator from '@/components/Quiz/QuizGenerator.vue'
import QuizSession from '@/components/Quiz/QuizSession.vue'
import ErrorBook from './ErrorBook.vue'
import SettingsPanel from './SettingsPanel.vue'
import { useQuizStore } from '@/stores/quiz'


const activeTab = ref<PanelTab>('quiz')
const quizStore = useQuizStore()
</script>
