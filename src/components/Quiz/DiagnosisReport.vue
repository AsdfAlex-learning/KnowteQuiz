<template>
  <div v-if="report" class="space-y-4">
    <!-- Summary -->
    <div class="bg-[var(--bg-elevated)] rounded-lg p-4">
      <h3 class="text-xs font-semibold uppercase tracking-wider text-[var(--text-muted)] mb-2">Diagnosis Summary</h3>
      <p class="text-sm text-[var(--text-primary)] leading-relaxed">
        {{ report.summary }}
      </p>
    </div>

    <!-- Overall level -->
    <div class="flex items-center gap-2 px-1">
      <span class="text-xs text-[var(--text-muted)]">Overall Level:</span>
      <span class="px-2 py-0.5 text-xs font-semibold rounded" :class="levelClass">
        {{ report.overall_level }}
      </span>
    </div>

    <!-- Blind spot cards -->
    <div v-if="report.blind_spots.length" class="space-y-2">
      <h3 class="text-xs font-semibold uppercase tracking-wider text-[var(--text-muted)]">Blind Spots</h3>
      <DiagnosisReportCard v-for="(spot, i) in report.blind_spots" :key="i" :spot="spot" />
    </div>

    <!-- Next steps -->
    <div v-if="report.next_steps.length" class="bg-[var(--bg-base)] rounded-lg p-3">
      <h3 class="text-xs font-semibold uppercase tracking-wider text-[var(--text-muted)] mb-2">Next Steps</h3>
      <ul class="space-y-1.5">
        <li
          v-for="(step, i) in report.next_steps"
          :key="i"
          class="flex items-start gap-2 text-sm text-[var(--text-secondary)]"
        >
          <span class="text-[var(--accent-purple)] mt-0.5 text-xs">{{ i + 1 }}.</span>
          <span>{{ step }}</span>
        </li>
      </ul>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { DiagnosisReport } from '@/types/diagnosis';
import DiagnosisReportCard from './DiagnosisReportCard.vue';

const props = defineProps<{
  report: DiagnosisReport;
}>();

const levelClass = computed(() => {
  switch (props.report.overall_level) {
    case 'beginner':
      return 'bg-[var(--color-error)]/20 text-[var(--color-error)]';
    case 'intermediate':
      return 'bg-[var(--accent-peach)]/20 text-[var(--accent-peach)]';
    case 'advanced':
      return 'bg-[var(--accent-blue)]/20 text-[var(--accent-blue)]';
    case 'expert':
      return 'bg-[var(--accent-green)]/20 text-[var(--accent-green)]';
    default:
      return 'bg-[var(--bg-active)] text-[var(--text-muted)]';
  }
});
</script>
