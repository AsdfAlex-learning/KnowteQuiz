<template>
  <div class="space-y-2">
    <textarea
      v-model="text"
      rows="3"
      :placeholder="t('exploration.answer_placeholder')"
      :disabled="busy"
      class="w-full resize-none px-3 py-2 text-[var(--text-sm)] bg-[var(--bg-base)] text-[var(--text-primary)] border border-[var(--border-default)] rounded-[var(--radius-md)] focus:border-[var(--border-focus)] focus:outline-none placeholder:text-[var(--text-faint)] disabled:opacity-60"
      @keydown.enter.exact.prevent="submit"
      @keydown.ctrl.enter.prevent="submit"
    ></textarea>
    <p class="text-[10px] text-[var(--text-faint)]">{{ t('exploration.command_hint') }}</p>
    <div class="flex items-center justify-between gap-3">
      <p class="text-[11px] text-[var(--text-faint)]">{{ t('exploration.answer_hint') }}</p>
      <button
        class="px-4 py-2 rounded-[var(--radius-md)] bg-[var(--accent-purple)] text-[var(--bg-base)] font-medium text-[var(--text-sm)] hover:bg-[var(--accent-purple)]/85 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        :disabled="busy || !text.trim()"
        @click="submit"
      >
        {{ busy ? t('exploration.thinking') : t('exploration.send') }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useExplorationStore } from '@/stores/exploration';
import { useI18n } from '@/composables/useI18n';
import { parseExplorationCommand } from '@/utils/explorationCommands';

defineProps<{
  busy?: boolean;
}>();

const explorationStore = useExplorationStore();
const { t } = useI18n();
const text = ref('');

async function submit() {
  const value = text.value.trim();
  if (!value || explorationStore.thinking) return;

  const command = parseExplorationCommand(value);
  let ok = false;
  switch (command.kind) {
    case 'answer':
      ok = await explorationStore.answer(command.text);
      break;
    case 'undo':
      ok = await explorationStore.undo();
      break;
    case 'redo':
      ok = await explorationStore.redo();
      break;
    case 'skip':
      ok = await explorationStore.skip();
      break;
    case 'end':
      ok = await explorationStore.finish();
      break;
    case 'btw':
      ok = await explorationStore.askAside(command.question);
      break;
  }
  if (ok) text.value = '';
}
</script>
