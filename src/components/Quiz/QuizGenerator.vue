<template>
  <div class="p-4 space-y-5">
    <div class="flex items-center justify-between">
      <h2 class="text-sm font-semibold text-[var(--text-primary)]">Generate Quiz</h2>
      <ModeToggle v-model="mode" />
    </div>

    <!-- Question types -->
    <div>
      <span class="text-xs text-[var(--text-muted)] mb-1.5 block">Question Types</span>
      <div class="flex flex-wrap gap-2">
        <button
          v-for="qt in questionTypes"
          :key="qt.value"
          class="px-2.5 py-1 text-xs rounded-md border transition-colors"
          :class="
            selectedTypes.includes(qt.value)
              ? 'bg-[var(--accent-purple)]/20 border-[var(--border-focus)] text-[var(--accent-purple)]'
              : 'bg-[var(--bg-base)] border-[var(--border-default)] text-[var(--text-muted)] hover:border-[var(--text-faint)]'
          "
          @click="toggleType(qt.value)"
        >
          {{ qt.label }}
        </button>
      </div>
    </div>

    <!-- Count slider -->
    <label class="block">
      <span class="text-xs text-[var(--text-muted)] mb-1 flex justify-between">
        Number of Questions
        <span class="text-[var(--text-primary)]">{{ count }}</span>
      </span>
      <input
        v-model.number="count"
        type="range"
        min="1"
        max="20"
        step="1"
        class="w-full accent-[var(--accent-purple)]"
      />
    </label>

    <!-- Difficulty -->
    <label class="block">
      <span class="text-xs text-[var(--text-muted)] mb-1 block">Difficulty</span>
      <select
        v-model="difficulty"
        class="w-full bg-[var(--bg-base)] border border-[var(--border-default)] rounded-md px-3 py-1.5 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--border-focus)] transition-colors"
      >
        <option value="easy">Easy</option>
        <option value="medium">Medium</option>
        <option value="hard">Hard</option>
      </select>
    </label>

    <!-- Language -->
    <label class="block">
      <span class="text-xs text-[var(--text-muted)] mb-1 block">Language</span>
      <select
        v-model="lang"
        class="w-full bg-[var(--bg-base)] border border-[var(--border-default)] rounded-md px-3 py-1.5 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--border-focus)] transition-colors"
      >
        <option value="en">English</option>
        <option value="zh">中文</option>
        <option value="ja">日本語</option>
        <option value="ko">한국어</option>
      </select>
    </label>

    <!-- Generate button -->
    <button
      class="w-full py-2.5 rounded-md text-sm font-semibold transition-colors btn-press"
      :class="
        quizStore.isGenerating
          ? 'bg-[var(--bg-active)] text-[var(--text-muted)] cursor-wait'
          : 'bg-[var(--accent-purple)] text-[var(--bg-base)] hover:bg-[var(--accent-lavender)]'
      "
      :disabled="quizStore.isGenerating || !notePath"
      @click="handleGenerate"
    >
      {{ generatingLabel }}
    </button>

    <p v-if="!notePath" class="text-[11px] text-[var(--text-faint)] text-center">
      Select a note to generate a quiz
    </p>

    <div
      v-if="quizStore.generatingError"
      class="rounded-md border border-[var(--color-error)]/40 bg-[var(--color-error)]/10 p-3"
    >
      <p class="text-xs font-medium text-[var(--color-error)]">Quiz generation failed</p>
      <p class="mt-1 text-xs leading-relaxed text-[var(--text-muted)]">
        {{ quizStore.generatingError }}
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useQuizStore } from '@/stores/quiz'
import { useSettingsStore } from '@/stores/settings'
import { useExplorerStore } from '@/stores/explorer'
import { useMistakeStore } from '@/stores/mistakes'
import ModeToggle from './ModeToggle.vue'
import type { QuizMode } from '@/stores/quiz'
import type { QuestionType, QuizDifficulty, QuizLanguage } from '@/types/quiz'

const quizStore = useQuizStore()
const settingsStore = useSettingsStore()
const explorerStore = useExplorerStore()
const mistakeStore = useMistakeStore()

const mode = ref<QuizMode>('basic')
const selectedTypes = ref<QuestionType[]>([])
const count = ref(5)
const difficulty = ref<QuizDifficulty>('medium')
const lang = ref<QuizLanguage>('zh')

const notePath = ref<string | null>(null)
const fallbackQuestionTypes: QuestionType[] = ['single', 'short']
const supportedQuestionTypes: QuestionType[] = ['single', 'multiple', 'short']
const supportedLanguages: QuizLanguage[] = ['zh', 'en', 'ja', 'ko']
const supportedDifficulties: QuizDifficulty[] = ['easy', 'medium', 'hard']

const generatingLabel = computed(() => {
  if (!quizStore.isGenerating) return 'Start Quiz'
  const phase = quizStore.generatingPhase
  if (phase === 'requesting_model') return 'Requesting model...'
  if (phase === 'parsing_response') return 'Parsing response...'
  return 'Generating...'
})

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
  quizStore.setMode(mode.value)
  mistakeStore.clearSaveState()

  await quizStore.startQuiz({
    path: notePath.value,
    types: [...selectedTypes.value],
    count: count.value,
    difficulty: difficulty.value,
    lang: lang.value,
  })
}

watch(() => explorerStore.selectedPath, (path) => {
  notePath.value = path
}, { immediate: true })

watch(() => settingsStore.settings.quiz, (defaults) => {
  mode.value = defaults.default_mode === 'advanced' ? 'advanced' : 'basic'
  selectedTypes.value = normalizeQuestionTypes(defaults.default_types)
  count.value = defaults.default_count
  lang.value = normalizeLanguage(defaults.default_language)
  difficulty.value = normalizeDifficulty(defaults.default_difficulty)
}, { immediate: true, deep: true })

function normalizeQuestionTypes(types: unknown): QuestionType[] {
  if (!Array.isArray(types)) return [...fallbackQuestionTypes]
  const filtered = types.filter((type): type is QuestionType =>
    supportedQuestionTypes.includes(type as QuestionType)
  )
  return filtered.length > 0 ? filtered : [...fallbackQuestionTypes]
}

function normalizeLanguage(value: string): QuizLanguage {
  return supportedLanguages.includes(value as QuizLanguage) ? value as QuizLanguage : 'zh'
}

function normalizeDifficulty(value: string): QuizDifficulty {
  return supportedDifficulties.includes(value as QuizDifficulty) ? value as QuizDifficulty : 'medium'
}
</script>
