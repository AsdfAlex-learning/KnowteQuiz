import type { QuizQuestion } from '../types/quiz'

function normalizeTextAnswer(answer: string): string {
  return answer.trim().replace(/\s+/g, ' ').toLowerCase()
}

export function choiceLettersFromAnswer(answer: string): string[] {
  const seen = new Set<string>()
  const letters: string[] = []
  const normalized = answer.toUpperCase()

  for (const match of normalized.matchAll(/(?:^|[^A-Z])([A-Z])(?=$|[^A-Z])/g)) {
    const letter = match[1]
    if (!seen.has(letter)) {
      seen.add(letter)
      letters.push(letter)
    }
  }

  return letters
}

function sameLetterSet(a: string[], b: string[]): boolean {
  if (a.length !== b.length) return false
  const bSet = new Set(b)
  return a.every((letter) => bSet.has(letter))
}

export function isChoiceLetterCorrect(correctAnswer: string, letter: string): boolean {
  return choiceLettersFromAnswer(correctAnswer).includes(letter.toUpperCase())
}

export function isQuizAnswerCorrect(question: QuizQuestion, userAnswer: string | undefined): boolean {
  if (!userAnswer) return false

  if (question.question_type === 'single' && question.options.length > 0) {
    const expected = choiceLettersFromAnswer(question.answer)
    const actual = choiceLettersFromAnswer(userAnswer)
    if (expected.length > 0 && actual.length > 0) {
      return sameLetterSet(expected, actual)
    }
  }

  return normalizeTextAnswer(userAnswer) === normalizeTextAnswer(question.answer)
}
