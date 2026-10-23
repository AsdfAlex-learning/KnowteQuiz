<template>
  <div class="max-w-lg mx-auto space-y-4">
    <div>
      <h2 class="text-[var(--text-sm)] font-semibold text-[var(--text-primary)]">
        {{ t('exploration.domains_title') }}
      </h2>
      <p class="mt-1 text-[var(--text-xs)] text-[var(--text-muted)] leading-relaxed">
        {{ t('exploration.domains_hint') }}
      </p>
    </div>

    <div class="space-y-2">
      <div
        v-for="(domain, index) in draft"
        :key="domain.id"
        class="rounded-[var(--radius-md)] border border-[var(--border-default)] bg-[var(--bg-elevated)] p-2.5 space-y-1"
      >
        <input
          v-model="draft[index].title"
          type="text"
          class="w-full px-2 py-1 text-[var(--text-sm)] bg-[var(--bg-base)] text-[var(--text-primary)] border border-[var(--border-default)] rounded focus:border-[var(--border-focus)] focus:outline-none"
          :aria-label="t('exploration.domain_name')"
        />
        <p class="text-[11px] text-[var(--text-faint)]">
          {{ t('exploration.domain_concept_count', { count: domain.concept_ids.length }) }}
        </p>
      </div>
    </div>

    <div class="flex flex-wrap items-center gap-2">
      <button
        class="px-4 py-2 rounded-[var(--radius-md)] bg-[var(--accent-purple)] text-[var(--bg-base)] font-medium text-[var(--text-sm)] hover:bg-[var(--accent-purple)]/85 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        :disabled="confirming || draft.length === 0"
        @click="confirm"
      >
        {{ t('exploration.domain_confirm') }}
      </button>
      <button
        v-if="draft.length > 1"
        class="px-4 py-2 rounded-[var(--radius-md)] border border-[var(--border-default)] text-[var(--text-sm)] text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] transition-colors disabled:opacity-50"
        :disabled="confirming"
        @click="mergeAll"
      >
        {{ t('exploration.domain_merge_all') }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useI18n } from '@/composables/useI18n';
import type { Domain } from '@/types/exploration';

const props = defineProps<{
  domains: Domain[];
  confirming?: boolean;
}>();

const emit = defineEmits<{
  confirm: [domains: Domain[]];
}>();

const { t } = useI18n();
const draft = ref<Domain[]>([]);

watch(
  () => props.domains,
  (domains) => {
    draft.value = domains.map((domain) => ({ ...domain, concept_ids: [...domain.concept_ids] }));
  },
  { immediate: true, deep: true }
);

function confirm() {
  emit(
    'confirm',
    draft.value.map((domain) => ({ ...domain, title: domain.title.trim() }))
  );
}

function mergeAll() {
  const conceptIds: string[] = [];
  for (const domain of draft.value) {
    for (const id of domain.concept_ids) {
      if (!conceptIds.includes(id)) conceptIds.push(id);
    }
  }
  emit('confirm', [
    {
      id: 'd1',
      title: t('exploration.all_domains'),
      concept_ids: conceptIds,
    },
  ]);
}
</script>
