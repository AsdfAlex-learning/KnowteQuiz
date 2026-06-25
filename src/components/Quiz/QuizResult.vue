<template>
  <div class="space-y-5">
    <!-- Score display -->
    <div class="text-center py-4">
      <div class="text-4xl font-bold" :class="scoreColor">
        {{ correctCount }} / {{ total }}
      </div>
      <p class="text-xs text-[var(--text-muted)] mt-1">
        {{ Math.round(scorePercent) }}% correct
      </p>
    </div>

    <!-- Question review -->
    <div class="space-y-3">
      <h3 class="text-xs font-semibold uppercase tracking-wider text-[var(--text-muted)]">
        Review
      </h3>
      <div
        v-for="(q, i) in questions"
        :key="q.id"
        class="bg-[var(--bg-elevated)] rounded-lg p-3 space-y-2 card-hover"
      >
        <div class="flex items-start gap-2">
          <span
            class="shrink-0 w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-bold mt-0.5"
            :class="isCorrect(q) ? 'bg-[var(--accent-green)] text-[var(--bg-base)]' : 'bg-[var(--color-error)] text-[var(--bg-base)]'"
          >
            {{ isCorrect(q) ? '\u2713' : '\u2717' }}
          </span>
          <div class="flex-1 min-w-0">
            <p class="text-sm text-[var(--text-primary)] leading-snug">{{ q.question }}</p>
            <div class="mt-1.5 flex items-center gap-3 text-xs">
              <span class="text-[var(--color-error)]">
                Yours: {{ formatUserAnswer(answers.get(q.id)) }}
              </span>
              <span class="text-[var(--accent-green)]">
                Correct: {{ q.answer }}
              </span>
            </div>
            <p v-if="!isCorrect(q) && q.explanation" class="text-xs text-[var(--text-muted)] mt-1 leading-relaxed">
              {{ q.explanation }}
            </p>
          </div>
        </div>

        <!-- Save mistake button for wrong answers -->
        <button
          v-if="!isCorrect(q) && !mistakeStore.isSaved(q.id)"
          class="text-[11px] text-[var(--accent-purple)] hover:text-[var(--accent-lavender)] transition-colors disabled:cursor-wait disabled:text-[var(--text-muted)]"
          :disabled="mistakeStore.isSaving(q.id)"
          @click="handleSaveMistake(q)"
        >
          {{ mistakeStore.isSaving(q.id) ? 'Saving...' : 'Save to Mistake Book' }}
        </button>
        <span
          v-else-if="!isCorrect(q) && mistakeStore.isSaved(q.id)"
          class="text-[11px] text-[var(--accent-green)]"
        >
          Saved &#10003;
        </span>
        <p v-if="mistakeStore.errorFor(q.id)" class="text-[11px] text-[var(--color-error)]">
          {{ mistakeStore.errorFor(q.id) }}
        </p>
      </div>
    </div>

    <!-- Actions -->
    <div class="space-y-2">
      <button
        class="w-full py-2.5 rounded-md text-sm font-semibold bg-[var(--accent-purple)] text-[var(--bg-base)] hover:bg-[var(--accent-lavender)] transition-colors btn-press"
        @click="$emit('newQuiz')"
      >
        Start New Quiz
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useQuizStore } from '@/stores/quiz'
import { useExplorerStore } from '@/stores/explorer'
import { useReaderStore } from '@/stores/reader'
import { useMistakeStore } from '@/stores/mistakes'
import { isQuizAnswerCorrect } from '@/utils/answer'
import type { QuizQuestion } from '@/types/quiz'
import type { DiagnosisReport } from '@/types/diagnosis'
import type { QuizMode } from '@/stores/quiz'
import type { DiagnosisContext, MistakeEntry } from '@/types/mistake'

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
const mistakeStore = useMistakeStore()

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
  if (scorePercent.value >= 80) return 'text-[var(--accent-green)]'
  if (scorePercent.value >= 50) return 'text-[var(--color-warning)]'
  return 'text-[var(--color-error)]'
})

function isCorrect(q: QuizQuestion): boolean {
  return isQuizAnswerCorrect(q, answers.value.get(q.id))
}

function formatUserAnswer(answer: string | undefined): string {
  if (!answer) return '\u2014'
  return answer
}

async function handleSaveMistake(q: QuizQuestion) {
  const userAnswer = answers.value.get(q.id) || ''
  const notePath = explorerStore.selectedPath || ''
  const noteTitle = readerStore.currentNote?.title || ''
  const questionReasoning = props.mode === 'advanced'
    ? quizStore.advancedReasoningFor(q.id) ?? props.reasoning
    : props.reasoning
  const questionDiagnosis = diagnosisFor(q.id)

  const entry: MistakeEntry = {
    id: crypto.randomUUID(),
    note_path: notePath,
    note_title: noteTitle,
    question: q.question,
    user_answer: userAnswer,
    correct_answer: q.answer,
    explanation: q.explanation,
    mode: props.mode,
    user_reasoning: questionReasoning || undefined,
    diagnosis: questionDiagnosis,
    created_at: new Date().toISOString(),
    review_count: 0,
  }

  await mistakeStore.saveEntry(q.id, entry)
}

function diagnosisFor(questionId: string): DiagnosisContext | undefined {
  if (props.mode !== 'advanced') return undefined
  return quizStore.diagnosisContextFor(questionId)
    ?? (props.diagnosisReport
      ? {
          rounds: 1,
          conversation: [],
          final_report: props.diagnosisReport,
        }
      : undefined)
}
</script>
