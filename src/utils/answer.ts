import type { QuizQuestion } from '../types/quiz';

export type QuizSubmissionMode = 'basic' | 'advanced';

function normalizeTextAnswer(answer: string): string {
  return answer.trim().replace(/\s+/g, ' ').toLowerCase();
}

function stripOptionLabel(option: string): string {
  return option.trim().replace(/^[A-Z]\s*[.)\]:\uFF1A\u3001]\s*/i, '');
}

export function choiceLettersFromAnswer(answer: string): string[] {
  const seen = new Set<string>();
  const letters: string[] = [];
  const normalized = answer.toUpperCase();

  for (const match of normalized.matchAll(/(?:^|[^A-Z])([A-Z])(?=$|[^A-Z])/g)) {
    const letter = match[1];
    if (!seen.has(letter)) {
      seen.add(letter);
      letters.push(letter);
    }
  }

  return letters;
}

function sameLetterSet(a: string[], b: string[]): boolean {
  if (a.length !== b.length) return false;
  const bSet = new Set(b);
  return a.every((letter) => bSet.has(letter));
}

export function isChoiceLetterCorrect(correctAnswer: string, letter: string): boolean {
  return choiceLettersFromAnswer(correctAnswer).includes(letter.toUpperCase());
}

function choiceLettersFromOptionText(correctAnswer: string, options: string[]): string[] {
  const normalizedFullAnswer = normalizeTextAnswer(correctAnswer);
  const fullAnswerMatches = choiceLettersMatchingOptionTexts(new Set([normalizedFullAnswer]), options);
  if (fullAnswerMatches.length > 0) {
    return fullAnswerMatches;
  }

  const normalizedAnswers = correctAnswer
    .split(/(?:[,;\uFF0C\uFF1B\u3001]|\s+and\s+)/i)
    .map(normalizeTextAnswer)
    .filter(Boolean);
  const answerSet = new Set(normalizedAnswers.length > 0 ? normalizedAnswers : [normalizeTextAnswer(correctAnswer)]);

  return choiceLettersMatchingOptionTexts(answerSet, options);
}

function choiceLettersMatchingOptionTexts(answerSet: Set<string>, options: string[]): string[] {
  return options
    .map((option, index) => ({
      letter: String.fromCharCode(65 + index),
      option: normalizeTextAnswer(option),
      text: normalizeTextAnswer(stripOptionLabel(option)),
    }))
    .filter(({ option, text }) => answerSet.has(option) || answerSet.has(text))
    .map(({ letter }) => letter);
}

export function correctChoiceLetters(question: QuizQuestion): string[] {
  if ((question.question_type !== 'single' && question.question_type !== 'multiple') || question.options.length === 0) {
    return [];
  }
  const expected = choiceLettersFromAnswer(question.answer);
  return expected.length > 0 ? expected : choiceLettersFromOptionText(question.answer, question.options);
}

export function isQuizAnswerCorrect(question: QuizQuestion, userAnswer: string | undefined): boolean {
  if (!userAnswer) return false;

  if ((question.question_type === 'single' || question.question_type === 'multiple') && question.options.length > 0) {
    const expectedFromOptionText = correctChoiceLetters(question);
    const actual = choiceLettersFromAnswer(userAnswer);
    if (expectedFromOptionText.length > 0 && actual.length > 0) {
      return sameLetterSet(expectedFromOptionText, actual);
    }
  }

  return normalizeTextAnswer(userAnswer) === normalizeTextAnswer(question.answer);
}

export function canSubmitQuizAnswer(
  question: QuizQuestion | null,
  selectedOptions: number[],
  shortAnswer: string,
  mode: QuizSubmissionMode,
  reasoning: string
): boolean {
  if (!question) return false;
  if (mode === 'advanced' && reasoning.trim().length === 0) return false;
  if (question.question_type === 'short') {
    return shortAnswer.trim().length > 0;
  }
  return selectedOptions.length > 0;
}
