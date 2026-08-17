<template>
  <div class="h-full flex flex-col">
    <div
      v-if="quizStore.hasSession && quizStore.generatingError"
      class="mx-4 mt-4 rounded-md border border-[var(--color-error)]/40 bg-[var(--color-error)]/10 p-3"
    >
      <p class="text-xs font-medium text-[var(--color-error)]">{{ t('common.error') }}</p>
      <p class="mt-1 text-xs leading-relaxed text-[var(--text-muted)]">
        {{ quizStore.generatingError }}
      </p>
    </div>

    <!-- Generating state -->
    <div v-if="quizStore.isGenerating" class="flex-1 flex flex-col items-center justify-center gap-3 p-4">
      <div class="w-8 h-8 border-2 border-[var(--border-focus)] border-t-transparent rounded-full animate-spin" />
      <p class="text-sm text-[var(--text-muted)]">Generating questions...</p>
      <p class="text-xs text-[var(--text-faint)]">{{ quizStore.questions.length }} loaded</p>
    </div>

    <!-- Answering state -->
    <div
      v-else-if="
        quizStore.hasQuestions && !quizStore.showResults && quizStore.quizState === 'answering' && currentQuestion
      "
      class="flex-1 flex flex-col overflow-y-auto p-4 space-y-4"
    >
      <QuestionCard
        :question="currentQuestion"
        :current-index="quizStore.currentIndex"
        :total="quizStore.questions.length"
        :selected-options="selectedOptions"
        :submitted="submitted"
        @select-option="handleSelectOption"
      />

      <!-- Short answer input -->
      <AnswerInput v-if="currentQuestion.question_type === 'short'" v-model="shortAnswer" :disabled="submitted" />

      <!-- Reasoning input (advanced mode) -->
      <ReasoningInput v-if="mode === 'advanced'" v-model="reasoning" :disabled="submitted" />

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
          {{ mode === 'advanced' ? t('quiz.submit_diagnosis') : t('quiz.submit') }}
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
            {{ quizStore.isLastQuestion ? t('quiz.result') : t('quiz.next') }}
          </button>
        </template>
      </div>
    </div>

    <!-- Diagnosing state (advanced mode) -->
    <div v-else-if="quizStore.quizState === 'diagnosing'" class="flex-1 flex flex-col overflow-y-auto p-4 space-y-4">
      <h3 class="text-xs font-semibold uppercase tracking-wider text-[var(--accent-purple)]">Diagnosis</h3>
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
          class="w-full py-2.5 rounded-md text-sm font-semibold bg-[var(--accent-purple)] text-[var(--bg-base)] hover:bg-[var(--accent-lavender)] transition-colors disabled:cursor-wait disabled:bg-[var(--bg-active)] disabled:text-[var(--text-muted)]"
          :disabled="
            currentQuestion
              ? mistakeStore.isSaving(currentQuestion.id) || mistakeStore.isSaved(currentQuestion.id)
              : true
          "
          @click="handleSaveMistakeFromDiagnosis"
        >
          {{
            currentQuestion && mistakeStore.isSaved(currentQuestion.id)
              ? 'Saved'
              : currentQuestion && mistakeStore.isSaving(currentQuestion.id)
                ? 'Saving...'
                : 'Save to Mistake Book'
          }}
        </button>
        <p
          v-if="currentQuestion && mistakeStore.errorFor(currentQuestion.id)"
          class="text-xs text-[var(--color-error)]"
        >
          {{ mistakeStore.errorFor(currentQuestion.id) }}
        </p>
        <button
          class="w-full py-2.5 rounded-md text-sm font-medium bg-[var(--bg-elevated)] text-[var(--text-primary)] hover:bg-[var(--bg-active)] transition-colors"
          @click="handleNextAfterReport"
        >
          {{ quizStore.isLastQuestion ? t('quiz.result') : t('quiz.next') }}
        </button>
      </div>
    </div>

    <!-- Result state -->
    <div v-else-if="quizStore.showResults" class="flex-1 overflow-y-auto p-4">
      <QuizResult
        :mode="mode"
        :reasoning="reasoning"
        :diagnosis-report="quizStore.diagnosisReport"
        @new-quiz="handleNewQuiz"
      />
    </div>

    <!-- Idle state -->
    <div v-else class="flex-1 flex items-center justify-center p-4">
      <p class="text-sm text-[var(--text-faint)]">{{ t('reader.no_content') }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useQuizStore } from '@/stores/quiz';
import { useExplorerStore } from '@/stores/explorer';
import { useReaderStore } from '@/stores/reader';
import { useMistakeStore } from '@/stores/mistakes';
import { useI18n } from '@/composables/useI18n';
import QuestionCard from './QuestionCard.vue';
import AnswerInput from './AnswerInput.vue';
import ReasoningInput from './ReasoningInput.vue';
import DiagnosisChat from './DiagnosisChat.vue';
import DiagnosisReportComponent from './DiagnosisReport.vue';
import QuizResult from './QuizResult.vue';
import type { QuizMode } from '@/stores/quiz';
import type { MistakeEntry } from '@/types/mistake';
import { canSubmitQuizAnswer } from '@/utils/answer';
import { createLocalId } from '@/utils/id';

const props = defineProps<{
  mode: QuizMode;
}>();

const quizStore = useQuizStore();
const explorerStore = useExplorerStore();
const readerStore = useReaderStore();
const mistakeStore = useMistakeStore();
const { t } = useI18n();

const selectedOptions = ref<number[]>([]);
const shortAnswer = ref('');
const reasoning = ref('');
const submitted = ref(false);
const diagnosisSubmitting = ref(false);

const currentQuestion = computed(() => quizStore.currentQuestion);

const canSubmit = computed(() => {
  return canSubmitQuizAnswer(
    currentQuestion.value,
    selectedOptions.value,
    shortAnswer.value,
    props.mode,
    reasoning.value
  );
});

function handleSelectOption(index: number) {
  if (!submitted.value) {
    if (currentQuestion.value?.question_type === 'multiple') {
      selectedOptions.value = selectedOptions.value.includes(index)
        ? selectedOptions.value.filter((value) => value !== index)
        : [...selectedOptions.value, index];
    } else {
      selectedOptions.value = [index];
    }
  }
}

function handleSubmit() {
  if (!currentQuestion.value || !canSubmit.value) return;

  const answer =
    currentQuestion.value.question_type === 'short'
      ? shortAnswer.value
      : selectedOptions.value
          .slice()
          .sort((a, b) => a - b)
          .map((index) => String.fromCharCode(65 + index))
          .join(',');

  quizStore.submitAnswer(currentQuestion.value.id, answer);
  submitted.value = true;

  if (props.mode === 'advanced') {
    startDiagnosis(answer);
  }
}

async function startDiagnosis(answer: string) {
  if (!currentQuestion.value) return;
  const notePath = explorerStore.selectedPath || '';
  await quizStore.startDiagnosis(
    currentQuestion.value.question,
    currentQuestion.value.answer,
    answer,
    reasoning.value,
    notePath
  );
  if (quizStore.quizState === 'answering' && quizStore.generatingError) {
    submitted.value = false;
  }
}

async function handleDiagnosisReply(text: string) {
  diagnosisSubmitting.value = true;
  try {
    await quizStore.continueDiagnosis(text);
  } finally {
    diagnosisSubmitting.value = false;
  }
}

async function handleEndDiagnosis() {
  await quizStore.finishDiagnosis();
}

function handleNext() {
  submitted.value = false;
  selectedOptions.value = [];
  shortAnswer.value = '';
  reasoning.value = '';

  if (quizStore.isLastQuestion) {
    quizStore.finishQuiz();
  } else {
    quizStore.nextQuestion();
  }
}

function handleNextAfterReport() {
  if (currentQuestion.value && quizStore.diagnosisReport) {
    quizStore.recordAdvancedContext(
      currentQuestion.value.id,
      reasoning.value,
      quizStore.diagnosisMessages,
      quizStore.diagnosisReport
    );
  }

  if (quizStore.isLastQuestion) {
    quizStore.finishQuiz();
  } else {
    submitted.value = false;
    selectedOptions.value = [];
    shortAnswer.value = '';
    reasoning.value = '';
    quizStore.clearDiagnosis();
    quizStore.nextQuestion();
  }
}

async function handleSaveMistakeFromDiagnosis() {
  if (!currentQuestion.value) return;
  const answer = quizStore.userAnswers.get(currentQuestion.value.id);
  const answerStr = answer ? answer : '';
  const notePath = explorerStore.selectedPath || '';
  const noteTitle = readerStore.currentNote?.title || '';

  const entry: MistakeEntry = {
    id: createLocalId('mistake'),
    note_path: notePath,
    note_title: noteTitle,
    question: currentQuestion.value.question,
    user_answer: answerStr,
    correct_answer: currentQuestion.value.answer,
    explanation: currentQuestion.value.explanation,
    mode: 'advanced',
    user_reasoning: reasoning.value || undefined,
    diagnosis: quizStore.diagnosisReport
      ? {
          rounds: quizStore.diagnosisMessages.length,
          conversation: quizStore.diagnosisMessages,
          final_report: quizStore.diagnosisReport,
        }
      : undefined,
    created_at: new Date().toISOString(),
    review_count: 0,
  };

  await mistakeStore.saveEntry(currentQuestion.value.id, entry);
}

function handleNewQuiz() {
  quizStore.resetQuiz();
  submitted.value = false;
  selectedOptions.value = [];
  shortAnswer.value = '';
  reasoning.value = '';
  quizStore.clearDiagnosis();
}
</script>
