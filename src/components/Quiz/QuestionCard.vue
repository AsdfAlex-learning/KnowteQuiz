<template>
  <div class="space-y-4">
    <!-- Progress bar -->
    <div class="flex items-center gap-3">
      <div class="flex-1 h-1.5 bg-[var(--bg-elevated)] rounded-full overflow-hidden">
        <div
          class="h-full bg-gradient-to-r from-[var(--accent-purple)] to-[var(--accent-lavender)] rounded-full transition-all duration-500 ease-out"
          :style="{ width: progressPercent + '%' }"
        />
      </div>
      <span class="text-[11px] text-[var(--text-faint)] tabular-nums shrink-0">
        {{ currentIndex + 1 }} / {{ total }}
      </span>
    </div>

    <!-- Question type badge -->
    <span
      class="inline-block px-2 py-0.5 text-[10px] font-semibold uppercase rounded"
      :class="typeBadgeClass"
    >
      {{ question.question_type }}
    </span>

    <!-- Question text -->
    <h2 class="text-sm font-medium text-[var(--text-primary)] leading-relaxed">
      {{ question.question }}
    </h2>

    <!-- Options (for choice questions) -->
    <div v-if="question.options.length > 0" class="space-y-2">
      <OptionCard
        v-for="(opt, i) in question.options"
        :key="i"
        :letter="String.fromCharCode(65 + i)"
        :text="opt"
        :state="getOptionState(i)"
        :disabled="submitted"
        @select="$emit('selectOption', i)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { QuizQuestion } from '@/types/quiz'
import OptionCard from './OptionCard.vue'
import type { OptionState } from './OptionCard.vue'
import { correctChoiceLetters } from '@/utils/answer'

const props = defineProps<{
  question: QuizQuestion
  currentIndex: number
  total: number
  selectedOptions: number[]
  submitted: boolean
}>()

defineEmits<{
  selectOption: [index: number]
}>()

const progressPercent = computed(() =>
  props.total > 0 ? ((props.currentIndex + 1) / props.total) * 100 : 0
)

const typeBadgeClass = computed(() => {
  switch (props.question.question_type) {
    case 'single':
      return 'bg-[var(--accent-blue)]/20 text-[var(--accent-blue)]'
    case 'multiple':
      return 'bg-[var(--accent-purple)]/20 text-[var(--accent-purple)]'
    case 'short':
      return 'bg-[var(--accent-green)]/20 text-[var(--accent-green)]'
    default:
      return 'bg-[var(--bg-active)] text-[var(--text-muted)]'
  }
})

function getOptionState(index: number): OptionState {
  if (!props.submitted) {
    return props.selectedOptions.includes(index) ? 'selected' : 'default'
  }
  const letter = String.fromCharCode(65 + index)
  const isCorrect = correctChoiceLetters(props.question).includes(letter)

  if (isCorrect) return 'correct'
  if (props.selectedOptions.includes(index)) return 'incorrect'
  return 'default'
}
</script>
