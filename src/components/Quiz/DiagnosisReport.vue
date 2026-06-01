<template>
  <div v-if="report" class="space-y-4">
    <!-- Summary -->
    <div class="bg-[#313244] rounded-lg p-4">
      <h3 class="text-xs font-semibold uppercase tracking-wider text-[#a6adc8] mb-2">
        Diagnosis Summary
      </h3>
      <p class="text-sm text-[#f8f8f2] leading-relaxed">
        {{ report.summary }}
      </p>
    </div>

    <!-- Overall level -->
    <div class="flex items-center gap-2 px-1">
      <span class="text-xs text-[#a6adc8]">Overall Level:</span>
      <span class="px-2 py-0.5 text-xs font-semibold rounded" :class="levelClass">
        {{ report.overall_level }}
      </span>
    </div>

    <!-- Blind spot cards -->
    <div v-if="report.blind_spots.length" class="space-y-2">
      <h3 class="text-xs font-semibold uppercase tracking-wider text-[#a6adc8]">
        Blind Spots
      </h3>
      <DiagnosisReportCard
        v-for="(spot, i) in report.blind_spots"
        :key="i"
        :spot="spot"
      />
    </div>

    <!-- Next steps -->
    <div v-if="report.next_steps.length" class="bg-[#1e1e2e] rounded-lg p-3">
      <h3 class="text-xs font-semibold uppercase tracking-wider text-[#a6adc8] mb-2">
        Next Steps
      </h3>
      <ul class="space-y-1.5">
        <li
          v-for="(step, i) in report.next_steps"
          :key="i"
          class="flex items-start gap-2 text-sm text-[#cdd6f4]"
        >
          <span class="text-[#cba6f7] mt-0.5 text-xs">{{ i + 1 }}.</span>
          <span>{{ step }}</span>
        </li>
      </ul>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { DiagnosisReport } from '@/types/diagnosis'
import DiagnosisReportCard from './DiagnosisReportCard.vue'

const props = defineProps<{
  report: DiagnosisReport
}>()

const levelClass = computed(() => {
  switch (props.report.overall_level) {
    case 'beginner':
      return 'bg-[#f38ba8]/20 text-[#f38ba8]'
    case 'intermediate':
      return 'bg-[#fab387]/20 text-[#fab387]'
    case 'advanced':
      return 'bg-[#89b4fa]/20 text-[#89b4fa]'
    case 'expert':
      return 'bg-[#a6e3a1]/20 text-[#a6e3a1]'
    default:
      return 'bg-[#45475a] text-[#a6adc8]'
  }
})
</script>
