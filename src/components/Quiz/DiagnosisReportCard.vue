<template>
  <div class="border border-[var(--border-default)] rounded-lg p-3 space-y-2.5 card-hover">
    <div class="flex items-center gap-2">
      <span
        class="px-2 py-0.5 text-[10px] font-semibold rounded"
        :class="severityClass"
      >
        {{ spot.severity }}
      </span>
      <span class="text-xs font-medium text-[var(--accent-purple)]">{{ spot.tag }}</span>
    </div>

    <p class="text-xs text-[var(--text-secondary)] leading-relaxed">
      {{ spot.description }}
    </p>

    <div v-if="spot.note_reference" class="bg-[var(--bg-base)] rounded-md px-2.5 py-1.5">
      <p class="text-[11px] text-[var(--text-faint)] italic">
        {{ spot.note_reference }}
      </p>
    </div>

    <div v-if="spot.suggestion" class="flex items-start gap-1.5">
      <span class="text-[var(--accent-green)] text-xs mt-0.5">&rarr;</span>
      <p class="text-xs text-[var(--accent-green)] leading-relaxed">
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
      return 'bg-[var(--color-error)]/20 text-[var(--color-error)]'
    case 'medium':
      return 'bg-[var(--accent-peach)]/20 text-[var(--accent-peach)]'
    case 'low':
      return 'bg-[var(--color-warning)]/20 text-[var(--color-warning)]'
    default:
      return 'bg-[var(--bg-active)] text-[var(--text-muted)]'
  }
})
</script>
