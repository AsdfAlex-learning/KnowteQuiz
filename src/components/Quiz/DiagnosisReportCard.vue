<template>
  <div class="border border-[#45475a] rounded-lg p-3 space-y-2.5">
    <div class="flex items-center gap-2">
      <span
        class="px-2 py-0.5 text-[10px] font-semibold rounded"
        :class="severityClass"
      >
        {{ spot.severity }}
      </span>
      <span class="text-xs font-medium text-[#cba6f7]">{{ spot.tag }}</span>
    </div>

    <p class="text-xs text-[#cdd6f4] leading-relaxed">
      {{ spot.description }}
    </p>

    <div v-if="spot.note_reference" class="bg-[#1e1e2e] rounded-md px-2.5 py-1.5">
      <p class="text-[11px] text-[#585b70] italic">
        {{ spot.note_reference }}
      </p>
    </div>

    <div v-if="spot.suggestion" class="flex items-start gap-1.5">
      <span class="text-[#a6e3a1] text-xs mt-0.5">→</span>
      <p class="text-xs text-[#a6e3a1] leading-relaxed">
        {{ spot.suggestion }}
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { BlindSpot } from '@/types/diagnosis'

const props = defineProps<{
  spot: BlindSpot
}>()

const severityClass = computed(() => {
  switch (props.spot.severity) {
    case 'high':
      return 'bg-[#f38ba8]/20 text-[#f38ba8]'
    case 'medium':
      return 'bg-[#fab387]/20 text-[#fab387]'
    case 'low':
      return 'bg-[#f9e2af]/20 text-[#f9e2af]'
    default:
      return 'bg-[#45475a] text-[#a6adc8]'
  }
})
</script>
