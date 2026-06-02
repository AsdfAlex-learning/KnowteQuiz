<template>
  <div class="space-y-4">
    <!-- Progress bar -->
    <div class="flex items-center gap-3">
      <div class="flex-1 h-1 bg-[#313244] rounded-full overflow-hidden">
        <div
          class="h-full bg-[#cba6f7] rounded-full transition-all duration-300"
          :style="{ width: progressPercent + '%' }"
        />
      </div>
      <span class="text-[11px] text-[#585b70] tabular-nums shrink-0">
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
    <h2 class="text-sm font-medium text-[#f8f8f2] leading-relaxed">
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

const props = defineProps<{
  question: QuizQuestion
  currentIndex: number
  total: number
  selectedOption: number | null
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
      return 'bg-[#89b4fa]/20 text-[#89b4fa]'
    case 'short':
      return 'bg-[#a6e3a1]/20 text-[#a6e3a1]'
    default:
      return 'bg-[#45475a] text-[#a6adc8]'
  }
})

function getOptionState(index: number): OptionState {
  if (!props.submitted) {
    return props.selectedOption === index ? 'selected' : 'default'
  }
  const letter = String.fromCharCode(65 + index)
  const correctAnswer = props.question.answer
  const isCorrect = Array.isArray(correctAnswer)
    ? correctAnswer.includes(letter)
    : correctAnswer === letter

  if (isCorrect) return 'correct'
  if (props.selectedOption === index) return 'incorrect'
  return 'default'
}
</script>
