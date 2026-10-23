<template>
  <div
    class="rounded-[var(--radius-md)] border border-dashed border-[var(--border-default)] bg-[var(--bg-base)] px-3 py-2 space-y-1.5"
  >
    <p class="text-[10px] font-semibold uppercase tracking-wider text-[var(--text-faint)]">
      {{ t('exploration.aside_label') }}
    </p>
    <p class="text-[var(--text-xs)] text-[var(--text-secondary)]">{{ aside.question }}</p>
    <div class="quiz-md text-[var(--text-sm)] text-[var(--text-primary)] leading-relaxed" v-html="html"></div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from '@/composables/useI18n';
import { renderQuizMarkdown } from '@/utils/markdown';
import type { AsideTurn } from '@/types/exploration';

const props = defineProps<{
  aside: AsideTurn;
}>();

const { t } = useI18n();
const html = computed(() => renderQuizMarkdown(props.aside.answer_md));
</script>
