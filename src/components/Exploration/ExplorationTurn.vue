<template>
  <div class="space-y-2.5">
    <div class="space-y-1">
      <p class="text-[10px] font-semibold uppercase tracking-wider text-[var(--text-faint)]">
        {{ conceptTitle }}
      </p>
      <div class="quiz-md text-[var(--text-sm)] text-[var(--text-primary)] leading-relaxed" v-html="questionHtml"></div>
    </div>

    <div v-if="group.answer" class="flex justify-end">
      <p
        class="max-w-[85%] rounded-[var(--radius-md)] bg-[var(--accent-purple)]/12 px-3 py-2 text-[var(--text-sm)] text-[var(--text-primary)] whitespace-pre-wrap"
      >
        {{ group.answer }}
      </p>
    </div>

    <div
      v-if="group.result"
      class="rounded-[var(--radius-md)] border border-[var(--border-subtle)] bg-[var(--bg-elevated)] px-3 py-2 space-y-1.5"
    >
      <span
        class="inline-block px-1.5 py-0.5 rounded text-[10px] font-semibold uppercase tracking-wider"
        :class="verdictClass"
      >
        {{ verdictLabel }}
      </span>
      <p v-if="group.result.hint" class="text-[var(--text-xs)] text-[var(--color-warning)]">
        <span class="font-medium">{{ t('exploration.hint_label') }}:</span>
        {{ group.result.hint }}
      </p>
      <div
        v-if="group.result.feedback_md"
        class="quiz-md text-[var(--text-sm)] text-[var(--text-secondary)] leading-relaxed"
        v-html="feedbackHtml"
      ></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from '@/composables/useI18n';
import { renderQuizMarkdown } from '@/utils/markdown';
import type { TurnGroup } from '@/types/exploration';

const props = defineProps<{
  group: TurnGroup;
  conceptTitle: string;
}>();

const { t } = useI18n();

const questionHtml = computed(() => renderQuizMarkdown(props.group.question.prompt_md));
const feedbackHtml = computed(() => (props.group.result ? renderQuizMarkdown(props.group.result.feedback_md) : ''));

const verdictLabel = computed(() => {
  switch (props.group.result?.verdict) {
    case 'correct':
      return t('exploration.verdict_correct');
    case 'partial':
      return t('exploration.verdict_partial');
    case 'wrong':
      return t('exploration.verdict_wrong');
    default:
      return t('exploration.verdict_unknown');
  }
});

const verdictClass = computed(() => {
  switch (props.group.result?.verdict) {
    case 'correct':
      return 'bg-[var(--color-success)]/15 text-[var(--color-success)]';
    case 'partial':
      return 'bg-[var(--color-warning)]/15 text-[var(--color-warning)]';
    case 'wrong':
      return 'bg-[var(--color-error)]/15 text-[var(--color-error)]';
    default:
      return 'bg-[var(--bg-active)] text-[var(--text-muted)]';
  }
});
</script>
