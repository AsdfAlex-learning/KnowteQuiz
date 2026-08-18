<template>
  <div class="space-y-4">
    <button
      class="text-xs text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors flex items-center gap-1"
      @click="$emit('back')"
    >
      <span class="text-sm">&larr;</span> Back to list
    </button>

    <div class="space-y-3">
      <div>
        <span
          class="inline-block px-1.5 py-0.5 text-[10px] font-semibold uppercase rounded mb-2"
          :class="
            mistake.mode === 'advanced'
              ? 'bg-[var(--accent-purple)]/20 text-[var(--accent-purple)]'
              : 'bg-[var(--accent-blue)]/20 text-[var(--accent-blue)]'
          "
        >
          {{ mistake.mode }}
        </span>
        <h3 class="text-sm font-medium text-[var(--text-primary)] leading-relaxed">
          {{ mistake.question }}
        </h3>
      </div>

      <div class="bg-[var(--bg-base)] rounded-lg p-3 space-y-2">
        <div class="flex items-center gap-2">
          <span class="text-xs text-[var(--color-error)] font-medium">{{ t('error_book.your_answer') }}:</span>
          <span class="text-sm text-[var(--text-primary)]">{{ mistake.user_answer }}</span>
        </div>
        <div class="flex items-center gap-2">
          <span class="text-xs text-[var(--accent-green)] font-medium">Correct:</span>
          <span class="text-sm text-[var(--text-primary)]">{{ mistake.correct_answer }}</span>
        </div>
      </div>

      <div class="bg-[var(--bg-base)] rounded-lg p-3">
        <h4 class="text-xs font-semibold text-[var(--text-muted)] uppercase tracking-wider mb-1">
          {{ t('error_book.explanation') }}
        </h4>
        <p class="text-sm text-[var(--text-primary)] leading-relaxed whitespace-pre-wrap">
          {{ mistake.explanation }}
        </p>
      </div>

      <div v-if="mistake.user_reasoning" class="bg-[var(--bg-base)] rounded-lg p-3">
        <h4 class="text-xs font-semibold text-[var(--text-muted)] uppercase tracking-wider mb-1">
          {{ t('error_book.reasoning') }}
        </h4>
        <p class="text-sm text-[var(--text-secondary)] leading-relaxed whitespace-pre-wrap italic">
          {{ mistake.user_reasoning }}
        </p>
      </div>

      <div v-if="parsedDiagnosis" class="bg-[var(--bg-base)] rounded-lg p-3 space-y-3">
        <h4 class="text-xs font-semibold text-[var(--accent-purple)] uppercase tracking-wider">Diagnosis</h4>

        <div v-if="parsedDiagnosis.summary" class="text-sm text-[var(--text-primary)] leading-relaxed">
          {{ parsedDiagnosis.summary }}
        </div>

        <div v-if="parsedDiagnosis.blind_spots?.length" class="space-y-2">
          <div
            v-for="(spot, i) in parsedDiagnosis.blind_spots"
            :key="i"
            class="border border-[var(--border-default)] rounded-md p-2.5"
          >
            <div class="flex items-center gap-2 mb-1">
              <span
                class="px-1.5 py-0.5 text-[10px] font-semibold rounded bg-[var(--color-error)]/20 text-[var(--color-error)]"
              >
                {{ spot.severity }}
              </span>
              <span class="text-xs font-medium text-[var(--accent-purple)]">{{ spot.tag }}</span>
            </div>
            <p class="text-xs text-[var(--text-secondary)] leading-relaxed">{{ spot.description }}</p>
            <p v-if="spot.note_reference" class="text-xs text-[var(--text-faint)] mt-1 italic">
              Ref: {{ spot.note_reference }}
            </p>
          </div>
        </div>

        <div v-if="parsedDiagnosis.overall_level" class="text-xs text-[var(--accent-peach)]">
          Level: {{ parsedDiagnosis.overall_level }}
        </div>

        <div v-if="parsedDiagnosis.next_steps?.length">
          <h5 class="text-xs text-[var(--text-muted)] mb-1">Next Steps</h5>
          <ul class="text-xs text-[var(--text-secondary)] space-y-0.5 list-disc list-inside">
            <li v-for="(step, i) in parsedDiagnosis.next_steps" :key="i">{{ step }}</li>
          </ul>
        </div>
      </div>
    </div>

    <button
      class="w-full py-2 rounded-md text-sm font-medium bg-[var(--bg-elevated)] text-[var(--accent-purple)] hover:bg-[var(--bg-active)] transition-colors"
      @click="$emit('openNote', mistake.note_path)"
    >
      {{ t('error_book.open_note') }}
    </button>
    <button
      class="px-3 py-1.5 rounded-lg text-xs font-medium border transition-colors"
      :class="
        isReviewing
          ? 'border-[var(--border-default)] text-[var(--text-muted)] cursor-wait'
          : 'border-[var(--accent-green)]/30 text-[var(--accent-green)] hover:bg-[var(--accent-green)]/10'
      "
      :disabled="isReviewing"
      @click="handleMarkReviewed"
    >
      {{
        isReviewing
          ? t('common.loading')
          : mistake.review_count > 0
            ? t('error_book.review_count', { count: mistake.review_count })
            : t('error_book.mark_reviewed')
      }}
    </button>
    <p v-if="reviewError" class="text-[11px] text-[var(--color-error)]">
      {{ reviewError }}
    </p>
    <span v-if="mistake.last_reviewed_at" class="text-[11px] text-[var(--text-faint)]">
      Last: {{ formatLastReviewed(mistake.last_reviewed_at) }}
    </span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from '@/composables/useI18n';
import type { MistakeEntry } from '@/types/mistake';
import type { DiagnosisReport } from '@/types/diagnosis';
import { useMistakeStore } from '@/stores/mistakes';

const { t } = useI18n();

const props = defineProps<{
  mistake: MistakeEntry;
}>();

defineEmits<{
  back: [];
  openNote: [path: string];
}>();

const mistakeStore = useMistakeStore();
const isReviewing = computed(() => mistakeStore.isReviewing(props.mistake.id));
const reviewError = computed(() => mistakeStore.reviewErrorFor(props.mistake.id));

async function handleMarkReviewed() {
  await mistakeStore.markReviewed(props.mistake.id);
}

function formatLastReviewed(dateStr: string): string {
  const d = new Date(dateStr);
  return d.toLocaleDateString(undefined, {
    month: 'short',
    day: 'numeric',
  });
}

const parsedDiagnosis = computed<DiagnosisReport | null>(() => {
  if (!props.mistake.diagnosis) return null;
  try {
    const diagnosis =
      typeof props.mistake.diagnosis === 'string' ? JSON.parse(props.mistake.diagnosis) : props.mistake.diagnosis;

    return diagnosis.final_report ?? diagnosis;
  } catch {
    return null;
  }
});
</script>
