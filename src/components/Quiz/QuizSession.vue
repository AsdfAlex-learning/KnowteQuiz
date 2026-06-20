<template>
  <div class="h-full flex flex-col">
    <!-- Generating state -->
    <div
      v-if="quizStore.isGenerating"
      class="flex-1 flex flex-col items-center justify-center gap-3 p-4"
    >
      <div class="w-8 h-8 border-2 border-[var(--border-focus)] border-t-transparent rounded-full animate-spin" />
      <p class="text-sm text-[var(--text-muted)]">Generating questions...</p>
      <p class="text-xs text-[var(--text-faint)]">{{ quizStore.questions.length }} loaded</p>
    </div>

    <!-- Answering state -->
    <div
      v-else-if="quizStore.hasQuestions && !quizStore.showResults && quizStore.quizState === 'answering' && currentQuestion"
      class="flex-1 flex flex-col overflow-y-auto p-4 space-y-4"
    >
      <QuestionCard
        :question="currentQuestion"
        :current-index="quizStore.currentIndex"
        :total="quizStore.questions.length"
        :selected-option="selectedOption"
        :submitted="submitted"
        @select-option="handleSelectOption"
      />

      <!-- Short answer input -->
      <AnswerInput
        v-if="currentQuestion.question_type === 'short'"
        v-model="shortAnswer"
        :disabled="submitted"
      />

      <!-- Reasoning input (advanced mode) -->
      <ReasoningInput
        v-if="mode === 'advanced'"
        v-model="reasoning"
        :disabled="submitted"
      />

      <!-- Submit / Next buttons -->
      <div class="pt-2 space-y-2">
          <button
            v-if="!submitted"
            class="w-full py-2.5 rounded-md text-sm font-semibold transition-colors btn-press"
          :class="
            canSubmit
              ? 'bg-[var(--accent-purple)] text-[var(--bg-base)] hover:bg-[var(--accent-lavender)]'
              : 'bg-[var(--bg-active)] text-[var(--text-muted)] cursor-not-allowed'
          "
          :disabled="!canSubmit"
          @click="handleSubmit"
        >
          {{ mode === 'advanced' ? 'Submit & Diagnose' : 'Submit' }}
        </button>

        <template v-else>
          <!-- Explanation after submit -->
          <div v-if="currentQuestion.explanation" class="bg-[var(--bg-base)] rounded-lg p-3">
            <p class="text-xs text-[var(--text-muted)] mb-1">Explanation</p>
            <p class="text-sm text-[var(--text-secondary)] leading-relaxed">
              {{ currentQuestion.explanation }}
            </p>
          </div>

          <button
            class="w-full py-2.5 rounded-md text-sm font-semibold bg-[var(--bg-elevated)] text-[var(--text-primary)] hover:bg-[var(--bg-active)] transition-colors"
            @click="handleNext"
          >
            {{ quizStore.isLastQuestion ? 'View Results' : 'Next Question' }}
          </button>
        </template>
      </div>
    </div>

    <!-- Diagnosing state (advanced mode) -->
    <div
      v-else-if="quizStore.quizState === 'diagnosing'"
      class="flex-1 flex flex-col overflow-y-auto p-4 space-y-4"
    >
      <h3 class="text-xs font-semibold uppercase tracking-wider text-[var(--accent-purple)]">
        Diagnosis
      </h3>
      <DiagnosisChat
        :messages="quizStore.diagnosisMessages"
        :active="true"
        :completed="false"
        :submitting="diagnosisSubmitting"
        @send-reply="handleDiagnosisReply"
        @end-diagnosis="handleEndDiagnosis"
      />
    </div>

    <!-- Report state (advanced mode) -->
    <div
      v-else-if="quizStore.quizState === 'report' && quizStore.diagnosisReport"
      class="flex-1 flex flex-col overflow-y-auto p-4 space-y-4"
    >
      <DiagnosisReportComponent :report="quizStore.diagnosisReport" />

      <div class="space-y-2 pt-2">
        <button
          class="w-full py-2.5 rounded-md text-sm font-semibold bg-[var(--accent-purple)] text-[var(--bg-base)] hover:bg-[var(--accent-lavender)] transition-colors"
          @click="handleSaveMistakeFromDiagnosis"
        >
          Save to Mistake Book
        </button>
        <button
          class="w-full py-2.5 rounded-md text-sm font-medium bg-[var(--bg-elevated)] text-[var(--text-primary)] hover:bg-[var(--bg-active)] transition-colors"
          @click="handleNextAfterReport"
        >
          {{ quizStore.isLastQuestion ? 'View Results' : 'Next Question' }}
        </button>
      </div>
    </div>

    <!-- Result state -->
    <div
      v-else-if="quizStore.showResults"
      class="flex-1 overflow-y-auto p-4"
    >
      <QuizResult :mode="mode" :reasoning="reasoning" :diagnosis-report="quizStore.diagnosisReport" @new-quiz="handleNewQuiz" />
    </div>

    <!-- Idle state -->
    <div
      v-else
      class="flex-1 flex items-center justify-center p-4"
    >
      <p class="text-sm text-[var(--text-faint)]">Configure and start a quiz above</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useQuizStore } from '@/stores/quiz'
import { useExplorerStore } from '@/stores/explorer'
import { useReaderStore } from '@/stores/reader'
import { saveMistake } from '@/services/mistake'
import QuestionCard from './QuestionCard.vue'
import AnswerInput from './AnswerInput.vue'
import ReasoningInput from './ReasoningInput.vue'
import DiagnosisChat from './DiagnosisChat.vue'
import DiagnosisReportComponent from './DiagnosisReport.vue'
import QuizResult from './QuizResult.vue'
import type { QuizMode } from '@/stores/quiz'
import type { MistakeEntry } from '@/types/mistake'

const props = defineProps<{
  mode: QuizMode
}>()

const quizStore = useQuizStore()
const explorerStore = useExplorerStore()
const readerStore = useReaderStore()

const selectedOption = ref<number | null>(null)
const shortAnswer = ref('')
const reasoning = ref('')
const submitted = ref(false)
const diagnosisSubmitting = ref(false)

const currentQuestion = computed(() => quizStore.currentQuestion)

const canSubmit = computed(() => {
  if (!currentQuestion.value) return false
  if (currentQuestion.value.question_type === 'short') {
    return shortAnswer.value.trim().length > 0
  }
  return selectedOption.value !== null
})

function handleSelectOption(index: number) {
  if (!submitted.value) {
    selectedOption.value = index
  }
}

function handleSubmit() {
  if (!currentQuestion.value || !canSubmit.value) return

  const answer = currentQuestion.value.question_type === 'short'
    ? shortAnswer.value
    : String.fromCharCode(65 + (selectedOption.value ?? 0))

  quizStore.submitAnswer(currentQuestion.value.id, answer)
  submitted.value = true

  if (props.mode === 'advanced') {
    startDiagnosis(answer)
  }
}

async function startDiagnosis(answer: string) {
  if (!currentQuestion.value) return
  const notePath = explorerStore.selectedPath || ''
  await quizStore.startDiagnosis(currentQuestion.value.question, answer, reasoning.value, notePath)
}

async function handleDiagnosisReply(text: string) {
  diagnosisSubmitting.value = true
  try {
    await quizStore.continueDiagnosis(text)
  } finally {
    diagnosisSubmitting.value = false
  }
}

async function handleEndDiagnosis() {
  await quizStore.finishDiagnosis()
}

function handleNext() {
  submitted.value = false
  selectedOption.value = null
  shortAnswer.value = ''
  reasoning.value = ''

  if (quizStore.isLastQuestion) {
    quizStore.finishQuiz()
  } else {
    quizStore.nextQuestion()
  }
}

function handleNextAfterReport() {
  submitted.value = false
  selectedOption.value = null
  shortAnswer.value = ''
  reasoning.value = ''
  quizStore.clearDiagnosis()

  if (quizStore.isLastQuestion) {
    quizStore.finishQuiz()
  } else {
    quizStore.nextQuestion()
  }
}

async function handleSaveMistakeFromDiagnosis() {
  if (!currentQuestion.value) return
  const answer = quizStore.userAnswers.get(currentQuestion.value.id)
  const answerStr = answer ? answer : ''
  const notePath = explorerStore.selectedPath || ''
  const noteTitle = readerStore.currentNote?.title || ''

  const entry: MistakeEntry = {
    id: crypto.randomUUID(),
    note_path: notePath,
    note_title: noteTitle,
    question: currentQuestion.value.question,
    user_answer: answerStr,
    correct_answer: currentQuestion.value.answer,
    explanation: currentQuestion.value.explanation,
    mode: 'advanced',
    user_reasoning: reasoning.value || undefined,
    diagnosis: quizStore.diagnosisReport ? {
      rounds: quizStore.diagnosisMessages.length,
      conversation: quizStore.diagnosisMessages,
      final_report: quizStore.diagnosisReport,
    } : undefined,
    created_at: new Date().toISOString(),
    review_count: 0,
  }

  await saveMistake(entry)
}

function handleNewQuiz() {
  quizStore.resetQuiz()
  submitted.value = false
  selectedOption.value = null
  shortAnswer.value = ''
  reasoning.value = ''
  quizStore.clearDiagnosis()
}
</script>
