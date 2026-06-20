import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { QuizQuestion, DiagnosisRound, DiagnosisReport } from '../types'
import { generateQuiz, submitAnswerAdvanced, diagnoseFollowUp } from '../services/quiz'
import type { QuizStreamParams } from '../types/quiz'
import { isQuizAnswerCorrect } from '../utils/answer'

export type QuizMode = 'basic' | 'advanced'
export type QuizState = 'idle' | 'generating' | 'answering' | 'diagnosing' | 'report' | 'result'

export const useQuizStore = defineStore('quiz', () => {
  const mode = ref<QuizMode>('basic')
  const quizState = ref<QuizState>('idle')
  const questions = ref<QuizQuestion[]>([])
  const currentIndex = ref(0)
  const answers = ref<Map<string, string>>(new Map())
  const userReasoning = ref('')
  const sessionId = ref<string | null>(null)
  const diagnosisMessages = ref<DiagnosisRound[]>([])
  const diagnosisReport = ref<DiagnosisReport | null>(null)
  const generatingError = ref<string | null>(null)

  const currentQuestion = computed(() => questions.value[currentIndex.value] ?? null)
  const totalQuestions = computed(() => questions.value.length)
  const isGenerating = computed(() => quizState.value === 'generating')
  const hasSession = computed(() => questions.value.length > 0)
  const hasQuestions = computed(() => questions.value.length > 0)
  const showResults = computed(() => quizState.value === 'result')
  const isLastQuestion = computed(() => currentIndex.value >= questions.value.length - 1)
  const userAnswers = computed(() => answers.value)
  const progress = computed(() => questions.value.length > 0 ? ((currentIndex.value + 1) / questions.value.length) * 100 : 0)
  const score = computed(() => {
    if (questions.value.length === 0) return 0
    let correct = 0
    for (const q of questions.value) {
      const userAns = answers.value.get(q.id)
      if (isQuizAnswerCorrect(q, userAns)) {
        correct++
      }
    }
    return Math.round((correct / questions.value.length) * 100)
  })

  function setMode(newMode: QuizMode) { mode.value = newMode }

  function reset() {
    quizState.value = 'idle'
    questions.value = []
    currentIndex.value = 0
    answers.value = new Map()
    userReasoning.value = ''
    sessionId.value = null
    diagnosisMessages.value = []
    diagnosisReport.value = null
    generatingError.value = null
  }

  function addQuestion(q: QuizQuestion) {
    questions.value.push(q)
  }

  function setAnswer(questionId: string, answer: string) {
    answers.value.set(questionId, answer)
  }

  function submitAnswer(questionId: string, answer: string) {
    setAnswer(questionId, answer)
  }

  function nextQuestion() {
    if (currentIndex.value < questions.value.length - 1) {
      currentIndex.value++
    }
  }

  function finishQuiz() {
    quizState.value = 'result'
  }

  function addDiagnosisMessage(round: DiagnosisRound) {
    diagnosisMessages.value.push(round)
  }

  function setDiagnosisReport(report: DiagnosisReport) {
    diagnosisReport.value = report
    quizState.value = 'report'
  }

  async function startQuiz(params: QuizStreamParams) {
    reset()
    quizState.value = 'generating'
    try {
      await generateQuiz(
        params,
        (q) => addQuestion(q),
        (total) => {
          quizState.value = 'answering'
        },
        (err) => {
          generatingError.value = err
          quizState.value = 'idle'
        }
      )
    } catch (e) {
      generatingError.value = String(e)
      quizState.value = 'idle'
    }
  }

  async function startDiagnosis(question: string, userAnswer: string, userReasoning: string, notePath: string) {
    quizState.value = 'diagnosing'
    try {
      const sid = await submitAnswerAdvanced(
        question, userAnswer, userReasoning, notePath,
        (data) => addDiagnosisMessage(data),
        (data) => {
          addDiagnosisMessage({ role: 'ai', content: data.question, blind_spots: data.blind_spots, follow_up: data.question })
        },
        (report) => setDiagnosisReport(report),
        (err) => { generatingError.value = err }
      )
      sessionId.value = sid
    } catch (e) {
      generatingError.value = String(e)
      quizState.value = 'answering'
    }
  }

  async function continueDiagnosis(userReply: string) {
    if (!sessionId.value) return
    try {
      await diagnoseFollowUp(
        sessionId.value, userReply,
        (data) => {
          addDiagnosisMessage({ role: 'user', content: userReply, blind_spots: [] })
          addDiagnosisMessage({ role: 'ai', content: data.question, blind_spots: data.blind_spots, follow_up: data.question })
        },
        (report) => setDiagnosisReport(report),
        (err) => { generatingError.value = err }
      )
    } catch (e) {
      generatingError.value = String(e)
    }
  }

  const resetQuiz = reset

  return {
    mode, quizState, questions, currentIndex, answers, userReasoning,
    sessionId, diagnosisMessages, diagnosisReport, generatingError,
    currentQuestion, totalQuestions, isGenerating, hasSession, hasQuestions,
    showResults, isLastQuestion, userAnswers, progress, score,
    setMode, reset, resetQuiz, addQuestion, setAnswer, submitAnswer, nextQuestion,
    finishQuiz, addDiagnosisMessage, setDiagnosisReport,
    startQuiz, startDiagnosis, continueDiagnosis,
  }
})
