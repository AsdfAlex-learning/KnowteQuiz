<template>
  <div class="space-y-5">
    <!-- Score display -->
    <div class="text-center py-4">
      <div class="text-4xl font-bold" :class="scoreColor">
        {{ correctCount }} / {{ total }}
      </div>
      <p class="text-xs text-[#a6adc8] mt-1">
        {{ Math.round(scorePercent) }}% correct
      </p>
    </div>

    <!-- Question review -->
    <div class="space-y-3">
      <h3 class="text-xs font-semibold uppercase tracking-wider text-[#a6adc8]">
        Review
      </h3>
      <div
        v-for="(q, i) in questions"
        :key="q.id"
        class="bg-[#313244] rounded-lg p-3 space-y-2"
      >
        <div class="flex items-start gap-2">
          <span
            class="shrink-0 w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-bold mt-0.5"
            :class="isCorrect(q) ? 'bg-[#a6e3a1] text-[#1e1e2e]' : 'bg-[#f38ba8] text-[#1e1e2e]'"
          >
            {{ isCorrect(q) ? '✓' : '✗' }}
          </span>
          <div class="flex-1 min-w-0">
            <p class="text-sm text-[#f8f8f2] leading-snug">{{ q.question }}</p>
            <div class="mt-1.5 flex items-center gap-3 text-xs">
              <span class="text-[#f38ba8]">
                Yours: {{ formatUserAnswer(answers.get(q.id)) }}
              </span>
              <span class="text-[#a6e3a1]">
                Correct: {{ q.answer }}
              </span>
            </div>
            <p v-if="!isCorrect(q) && q.explanation" class="text-xs text-[#a6adc8] mt-1 leading-relaxed">
              {{ q.explanation }}
            </p>
          </div>
        </div>

        <!-- Save mistake button for wrong answers -->
        <button
          v-if="!isCorrect(q) && !savedIds.has(q.id)"
          class="text-[11px] text-[#cba6f7] hover:text-[#b4befe] transition-colors"
          @click="handleSaveMistake(q)"
        >
          Save to Mistake Book
        </button>
        <span
          v-else-if="!isCorrect(q) && savedIds.has(q.id)"
          class="text-[11px] text-[#a6e3a1]"
        >
          Saved ✓
        </span>
      </div>
    </div>

    <!-- Actions -->
    <div class="space-y-2">
      <button
        class="w-full py-2.5 rounded-md text-sm font-semibold bg-[#cba6f7] text-[#1e1e2e] hover:bg-[#b4befe] transition-colors"
        @click="$emit('newQuiz')"
      >
        Start New Quiz
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useQuizStore } from '@/stores/quiz'
import { useExplorerStore } from '@/stores/explorer'
import { useReaderStore } from '@/stores/reader'
import { saveMistake } from '@/services/mistake'
import type { QuizQuestion } from '@/types/quiz'
import type { DiagnosisReport } from '@/types/diagnosis'
import type { QuizMode } from '@/stores/quiz'
import type { MistakeEntry } from '@/types/mistake'

const props = defineProps<{
  mode: QuizMode
  reasoning: string
  diagnosisReport: DiagnosisReport | null
}>()

const emit = defineEmits<{
  newQuiz: []
}>()

const quizStore = useQuizStore()
const explorerStore = useExplorerStore()
const readerStore = useReaderStore()
const savedIds = ref(new Set<string>())

const questions = computed(() => quizStore.questions)
const answers = computed(() => quizStore.userAnswers)
const total = computed(() => questions.value.length)

const correctCount = computed(() =>
  questions.value.filter((q) => isCorrect(q)).length
)

const scorePercent = computed(() =>
  total.value > 0 ? (correctCount.value / total.value) * 100 : 0
)

const scoreColor = computed(() => {
  if (scorePercent.value >= 80) return 'text-[#a6e3a1]'
  if (scorePercent.value >= 50) return 'text-[#f9e2af]'
  return 'text-[#f38ba8]'
})

function isCorrect(q: QuizQuestion): boolean {
  const userAnswer = answers.value.get(q.id)
  if (!userAnswer) return false
  return userAnswer.toLowerCase().trim() === q.answer.toLowerCase().trim()
}

function formatUserAnswer(answer: string | undefined): string {
  if (!answer) return '—'
  return answer
}

async function handleSaveMistake(q: QuizQuestion) {
  const userAnswer = answers.value.get(q.id) || ''
  const notePath = explorerStore.selectedPath || ''
  const noteTitle = readerStore.currentNote?.title || ''

  const entry: MistakeEntry = {
    id: crypto.randomUUID(),
    note_path: notePath,
    note_title: noteTitle,
    question: q.question,
    user_answer: userAnswer,
    correct_answer: q.answer,
    explanation: q.explanation,
    mode: props.mode,
    user_reasoning: props.reasoning || undefined,
    diagnosis: props.diagnosisReport ? {
      rounds: 1,
      conversation: [],
      final_report: props.diagnosisReport,
    } : undefined,
    created_at: new Date().toISOString(),
    review_count: 0,
  }

  await saveMistake(entry)
  savedIds.value.add(q.id)
}
</script>
