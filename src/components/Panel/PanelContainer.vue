<template>
  <div class="h-full flex flex-col bg-[var(--bg-base)]">
    <PanelTabs v-model="activeTab" @open-settings="showSettings = true" />
    <div class="flex-1 overflow-y-auto">
      <div v-if="activeTab === 'quiz'" class="h-full flex flex-col">
        <QuizGenerator v-if="!quizStore.hasSession && !quizStore.isGenerating" />
        <QuizSession v-else :mode="quizStore.mode" />
      </div>
      <ErrorBook v-else-if="activeTab === 'mistakes'" />
    </div>
    <SettingsModal v-model="showSettings" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import PanelTabs from './PanelTabs.vue';
import QuizGenerator from '@/components/Quiz/QuizGenerator.vue';
import QuizSession from '@/components/Quiz/QuizSession.vue';
import ErrorBook from './ErrorBook.vue';
import SettingsModal from './SettingsModal.vue';
import { useQuizStore } from '@/stores/quiz';

const activeTab = ref<'quiz' | 'mistakes'>('quiz');
const showSettings = ref(false);
const quizStore = useQuizStore();
</script>
