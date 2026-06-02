<template>
  <div class="h-full flex flex-col">
    <!-- Generating state -->
    <div
      v-if="quizStore.isGenerating"
      class="flex-1 flex flex-col items-center justify-center gap-3 p-4"
    >
      <div class="w-8 h-8 border-2 border-[#cba6f7] border-t-transparent rounded-full animate-spin" />
      <p class="text-sm text-[#a6adc8]">Generating questions...</p>
      <p class="text-xs text-[#585b70]">{{ quizStore.questions.length }} loaded</p>
    </div>

    <!-- Answering state -->
    <div
      v-else-if="quizStore.hasQuestions && !quizStore.showResults && phase === 'answering' && currentQuestion"
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
          class="w-full py-2.5 rounded-md text-sm font-semibold transition-colors"
          :class="
            canSubmit
              ? 'bg-[#cba6f7] text-[#1e1e2e] hover:bg-[#b4befe]'
              : 'bg-[#45475a] text-[#a6adc8] cursor-not-allowed'
          "
          :disabled="!canSubmit"
          @click="handleSubmit"
        >
          {{ mode === 'advanced' ? 'Submit & Diagnose' : 'Submit' }}
        </button>

        <template v-else>
          <!-- Explanation after submit -->
          <div v-if="currentQuestion.explanation" class="bg-[#1e1e2e] rounded-lg p-3">
            <p class="text-xs text-[#a6adc8] mb-1">Explanation</p>
            <p class="text-sm text-[#cdd6f4] leading-relaxed">
              {{ currentQuestion.explanation }}
            </p>
          </div>

          <button
            class="w-full py-2.5 rounded-md text-sm font-semibold bg-[#313244] text-[#f8f8f2] hover:bg-[#45475a] transition-colors"
            @click="handleNext"
          >
            {{ quizStore.isLastQuestion ? 'View Results' : 'Next Question' }}
          </button>
        </template>
      </div>
    </div>

    <!-- Diagnosing state (advanced mode) -->
    <div
      v-else-if="phase === 'diagnosing'"
      class="flex-1 flex flex-col overflow-y-auto p-4 space-y-4"
    >
      <h3 class="text-xs font-semibold uppercase tracking-wider text-[#cba6f7]">
        Diagnosis
      </h3>
      <DiagnosisChat
        :messages="diagnosisMessages"
        :active="true"
        :completed="false"
        :submitting="diagnosisSubmitting"
        @send-reply="handleDiagnosisReply"
        @end-diagnosis="handleEndDiagnosis"
      />
    </div>

    <!-- Report state (advanced mode) -->
    <div
      v-else-if="phase === 'report' && diagnosisReport"
      class="flex-1 flex flex-col overflow-y-auto p-4 space-y-4"
    >
      <DiagnosisReportComponent :report="diagnosisReport" />

      <div class="space-y-2 pt-2">
        <button
          class="w-full py-2.5 rounded-md text-sm font-semibold bg-[#cba6f7] text-[#1e1e2e] hover:bg-[#b4befe] transition-colors"
          @click="handleSaveMistakeFromDiagnosis"
        >
          Save to Mistake Book
        </button>
        <button
          class="w-full py-2.5 rounded-md text-sm font-medium bg-[#313244] text-[#f8f8f2] hover:bg-[#45475a] transition-colors"
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
      <QuizResult :mode="mode" :reasoning="reasoning" :diagnosis-report="diagnosisReport" @new-quiz="handleNewQuiz" />
    </div>

    <!-- Idle state -->
    <div
      v-else
      class="flex-1 flex items-center justify-center p-4"
    >
      <p class="text-sm text-[#585b70]">Configure and start a quiz above</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useQuizStore } from '@/stores/quiz'
import { useExplorerStore } from '@/stores/explorer'
import { useReaderStore } from '@/stores/reader'
import { submitAnswerAdvanced, diagnoseFollowUp, generateDiagnosisReport } from '@/services/quiz'
import { saveMistake } from '@/services/mistake'
import QuestionCard from './QuestionCard.vue'
import AnswerInput from './AnswerInput.vue'
import ReasoningInput from './ReasoningInput.vue'
import DiagnosisChat from './DiagnosisChat.vue'
import DiagnosisReportComponent from './DiagnosisReport.vue'
import QuizResult from './QuizResult.vue'
import type { DiagnosisRound, DiagnosisReport as DiagnosisReportType } from '@/types/diagnosis'
import type { QuizMode } from '@/stores/quiz'
import type { MistakeEntry } from '@/types/mistake'

const props = defineProps<{
  mode: QuizMode
}>()

const quizStore = useQuizStore()
const explorerStore = useExplorerStore()
const readerStore = useReaderStore()

type Phase = 'answering' | 'diagnosing' | 'report'
const phase = ref<Phase>('answering')
const selectedOption = ref<number | null>(null)
const shortAnswer = ref('')
const reasoning = ref('')
const submitted = ref(false)
const diagnosisSubmitting = ref(false)
const diagnosisMessages = ref<DiagnosisRound[]>([])
const diagnosisReport = ref<DiagnosisReportType | null>(null)
const diagSessionId = ref<string | null>(null)

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
  phase.value = 'diagnosing'
  diagnosisMessages.value = []

  try {
    const notePath = explorerStore.selectedPath || ''
    const sid = await submitAnswerAdvanced(
      currentQuestion.value.question,
      answer,
      reasoning.value,
      notePath,
      (data) => diagnosisMessages.value.push(data),
      (data) => {
        diagnosisMessages.value.push({ role: 'user', content: '', blind_spots: [] })
        diagnosisMessages.value.push({ role: 'ai', content: data.question, blind_spots: data.blind_spots, follow_up: data.question })
      },
      (report) => {
        diagnosisReport.value = report
        phase.value = 'report'
      },
      (err) => console.error('Diagnosis error:', err)
    )
    diagSessionId.value = sid
  } catch (err) {
    console.error('Failed to start diagnosis:', err)
    phase.value = 'answering'
  }
}

async function handleDiagnosisReply(text: string) {
  if (!diagSessionId.value) return
  diagnosisSubmitting.value = true

  try {
    await diagnoseFollowUp(
      diagSessionId.value,
      text,
      (data) => {
        diagnosisMessages.value.push({ role: 'user', content: text, blind_spots: [] })
        diagnosisMessages.value.push({ role: 'ai', content: data.question, blind_spots: data.blind_spots, follow_up: data.question })
      },
      (report) => {
        diagnosisReport.value = report
        phase.value = 'report'
        diagnosisSubmitting.value = false
      },
      (err) => {
        console.error('Follow-up error:', err)
        diagnosisSubmitting.value = false
      }
    )
  } catch {
    diagnosisSubmitting.value = false
  }
}

async function handleEndDiagnosis() {
  if (!diagSessionId.value) return

  try {
    const report = await generateDiagnosisReport(diagSessionId.value)
    diagnosisReport.value = report
    phase.value = 'report'
  } catch (err) {
    console.error('Failed to generate report:', err)
  }
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
  diagnosisMessages.value = []
  diagnosisReport.value = null
  diagSessionId.value = null
  phase.value = 'answering'

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
    diagnosis: diagnosisReport.value ? {
      rounds: diagnosisMessages.value.length,
      conversation: diagnosisMessages.value,
      final_report: diagnosisReport.value,
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
  phase.value = 'answering'
  diagnosisMessages.value = []
  diagnosisReport.value = null
  diagSessionId.value = null
}
</script>
