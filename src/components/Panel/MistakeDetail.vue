<template>
  <div class="space-y-4">
    <button
      class="text-xs text-[#a6adc8] hover:text-[#f8f8f2] transition-colors flex items-center gap-1"
      @click="$emit('back')"
    >
      <span class="text-sm">←</span> Back to list
    </button>

    <div class="space-y-3">
      <div>
        <span
          class="inline-block px-1.5 py-0.5 text-[10px] font-semibold uppercase rounded mb-2"
          :class="
            mistake.mode === 'advanced'
              ? 'bg-[#cba6f7]/20 text-[#cba6f7]'
              : 'bg-[#89b4fa]/20 text-[#89b4fa]'
          "
        >
          {{ mistake.mode }}
        </span>
        <h3 class="text-sm font-medium text-[#f8f8f2] leading-relaxed">
          {{ mistake.question }}
        </h3>
      </div>

      <div class="bg-[#1e1e2e] rounded-lg p-3 space-y-2">
        <div class="flex items-center gap-2">
          <span class="text-xs text-[#f38ba8] font-medium">Your answer:</span>
          <span class="text-sm text-[#f8f8f2]">{{ mistake.user_answer }}</span>
        </div>
        <div class="flex items-center gap-2">
          <span class="text-xs text-[#a6e3a1] font-medium">Correct:</span>
          <span class="text-sm text-[#f8f8f2]">{{ mistake.correct_answer }}</span>
        </div>
      </div>

      <div class="bg-[#1e1e2e] rounded-lg p-3">
        <h4 class="text-xs font-semibold text-[#a6adc8] uppercase tracking-wider mb-1">
          Explanation
        </h4>
        <p class="text-sm text-[#f8f8f2] leading-relaxed whitespace-pre-wrap">
          {{ mistake.explanation }}
        </p>
      </div>

      <div v-if="mistake.user_reasoning" class="bg-[#1e1e2e] rounded-lg p-3">
        <h4 class="text-xs font-semibold text-[#a6adc8] uppercase tracking-wider mb-1">
          Your Reasoning
        </h4>
        <p class="text-sm text-[#cdd6f4] leading-relaxed whitespace-pre-wrap italic">
          {{ mistake.user_reasoning }}
        </p>
      </div>

      <div v-if="parsedDiagnosis" class="bg-[#1e1e2e] rounded-lg p-3 space-y-3">
        <h4 class="text-xs font-semibold text-[#cba6f7] uppercase tracking-wider">
          Diagnosis
        </h4>

        <div v-if="parsedDiagnosis.summary" class="text-sm text-[#f8f8f2] leading-relaxed">
          {{ parsedDiagnosis.summary }}
        </div>

        <div v-if="parsedDiagnosis.blind_spots?.length" class="space-y-2">
          <div
            v-for="(spot, i) in parsedDiagnosis.blind_spots"
            :key="i"
            class="border border-[#45475a] rounded-md p-2.5"
          >
            <div class="flex items-center gap-2 mb-1">
              <span class="px-1.5 py-0.5 text-[10px] font-semibold rounded bg-[#f38ba8]/20 text-[#f38ba8]">
                {{ spot.severity }}
              </span>
              <span class="text-xs font-medium text-[#cba6f7]">{{ spot.tag }}</span>
            </div>
            <p class="text-xs text-[#cdd6f4] leading-relaxed">{{ spot.description }}</p>
            <p v-if="spot.note_reference" class="text-xs text-[#585b70] mt-1 italic">
              Ref: {{ spot.note_reference }}
            </p>
          </div>
        </div>

        <div v-if="parsedDiagnosis.overall_level" class="text-xs text-[#fab387]">
          Level: {{ parsedDiagnosis.overall_level }}
        </div>

        <div v-if="parsedDiagnosis.next_steps?.length">
          <h5 class="text-xs text-[#a6adc8] mb-1">Next Steps</h5>
          <ul class="text-xs text-[#cdd6f4] space-y-0.5 list-disc list-inside">
            <li v-for="(step, i) in parsedDiagnosis.next_steps" :key="i">{{ step }}</li>
          </ul>
        </div>
      </div>
    </div>

    <button
      class="w-full py-2 rounded-md text-sm font-medium bg-[#313244] text-[#cba6f7] hover:bg-[#45475a] transition-colors"
      @click="$emit('openNote', mistake.note_path)"
    >
      Open Note
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { MistakeEntry } from '@/types/mistake'
import type { DiagnosisReport } from '@/types/diagnosis'

const props = defineProps<{
  mistake: MistakeEntry
}>()

defineEmits<{
  back: []
  openNote: [path: string]
}>()

const parsedDiagnosis = computed<DiagnosisReport | null>(() => {
  if (!props.mistake.diagnosis) return null
  try {
    return typeof props.mistake.diagnosis === 'string'
      ? JSON.parse(props.mistake.diagnosis)
      : props.mistake.diagnosis
  } catch {
    return null
  }
})
</script>
