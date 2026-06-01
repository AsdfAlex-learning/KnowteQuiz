<template>
  <div class="p-4 space-y-5">
    <div class="flex items-center justify-between">
      <h2 class="text-sm font-semibold text-[#f8f8f2]">Generate Quiz</h2>
      <ModeToggle v-model="mode" />
    </div>

    <!-- Question types -->
    <div>
      <span class="text-xs text-[#a6adc8] mb-1.5 block">Question Types</span>
      <div class="flex flex-wrap gap-2">
        <button
          v-for="qt in questionTypes"
          :key="qt.value"
          class="px-2.5 py-1 text-xs rounded-md border transition-colors"
          :class="
            selectedTypes.includes(qt.value)
              ? 'bg-[#cba6f7]/20 border-[#cba6f7] text-[#cba6f7]'
              : 'bg-[#1e1e2e] border-[#45475a] text-[#a6adc8] hover:border-[#585b70]'
          "
          @click="toggleType(qt.value)"
        >
          {{ qt.label }}
        </button>
      </div>
    </div>

    <!-- Count slider -->
    <label class="block">
      <span class="text-xs text-[#a6adc8] mb-1 flex justify-between">
        Number of Questions
        <span class="text-[#f8f8f2]">{{ count }}</span>
      </span>
      <input
        v-model.number="count"
        type="range"
        min="1"
        max="20"
        step="1"
        class="w-full accent-[#cba6f7]"
      />
    </label>

    <!-- Difficulty -->
    <label class="block">
      <span class="text-xs text-[#a6adc8] mb-1 block">Difficulty</span>
      <select
        v-model="difficulty"
        class="w-full bg-[#1e1e2e] border border-[#45475a] rounded-md px-3 py-1.5 text-sm text-[#f8f8f2] focus:outline-none focus:border-[#cba6f7] transition-colors"
      >
        <option value="easy">Easy</option>
        <option value="medium">Medium</option>
        <option value="hard">Hard</option>
      </select>
    </label>

    <!-- Language -->
    <label class="block">
      <span class="text-xs text-[#a6adc8] mb-1 block">Language</span>
      <select
        v-model="lang"
        class="w-full bg-[#1e1e2e] border border-[#45475a] rounded-md px-3 py-1.5 text-sm text-[#f8f8f2] focus:outline-none focus:border-[#cba6f7] transition-colors"
      >
        <option value="en">English</option>
        <option value="zh">中文</option>
        <option value="ja">日本語</option>
        <option value="ko">한국어</option>
      </select>
    </label>

    <!-- Generate button -->
    <button
      class="w-full py-2.5 rounded-md text-sm font-semibold transition-colors"
      :class="
        quizStore.isGenerating
          ? 'bg-[#45475a] text-[#a6adc8] cursor-wait'
          : 'bg-[#cba6f7] text-[#1e1e2e] hover:bg-[#b4befe]'
      "
      :disabled="quizStore.isGenerating || !notePath"
      @click="handleGenerate"
    >
      {{ quizStore.isGenerating ? 'Generating...' : 'Start Quiz' }}
    </button>

    <p v-if="!notePath" class="text-[11px] text-[#585b70] text-center">
      Select a note to generate a quiz
    </p>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useQuizStore } from '@/stores/quiz'
import { useSettingsStore } from '@/stores/settings'
import { useExplorerStore } from '@/stores/explorer'
import ModeToggle from './ModeToggle.vue'
import type { QuizMode } from '@/stores/quiz'
import type { QuestionType, QuizDifficulty, QuizLanguage } from '@/types/quiz'

const quizStore = useQuizStore()
const settingsStore = useSettingsStore()
const explorerStore = useExplorerStore()

const mode = ref<QuizMode>('basic')
const selectedTypes = ref<QuestionType[]>([])
const count = ref(5)
const difficulty = ref<QuizDifficulty>('medium')
const lang = ref<QuizLanguage>('zh')

const notePath = ref<string | null>(null)

const questionTypes: { value: QuestionType; label: string }[] = [
  { value: 'single', label: 'Single Choice' },
  { value: 'multiple', label: 'Multiple Choice' },
  { value: 'short', label: 'Short Answer' },
]

function toggleType(type: QuestionType) {
  const idx = selectedTypes.value.indexOf(type)
  if (idx >= 0) {
    if (selectedTypes.value.length > 1) {
      selectedTypes.value.splice(idx, 1)
    }
  } else {
    selectedTypes.value.push(type)
  }
}

async function handleGenerate() {
  if (!notePath.value || quizStore.isGenerating) return
  quizStore.resetQuiz()

  await quizStore.startQuiz({
    path: notePath.value,
    types: selectedTypes.value,
    count: count.value,
    difficulty: difficulty.value,
    lang: lang.value,
  })
}

onMounted(() => {
  const defaults = settingsStore.settings.quiz
  selectedTypes.value = defaults.default_types as QuestionType[]
  count.value = defaults.default_count
  lang.value = defaults.default_language as QuizLanguage
  notePath.value = explorerStore.selectedPath
})
</script>
