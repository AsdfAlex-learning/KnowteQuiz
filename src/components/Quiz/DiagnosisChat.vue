<template>
  <div class="space-y-3">
    <div
      v-for="(msg, i) in messages"
      :key="i"
      class="flex"
      :class="msg.role === 'ai' ? 'justify-start' : 'justify-end'"
    >
      <div
        class="max-w-[85%] rounded-lg px-3 py-2 text-sm leading-relaxed"
        :class="
          msg.role === 'ai'
            ? 'bg-[var(--bg-elevated)] text-[var(--text-secondary)]'
            : 'bg-[var(--accent-purple)] text-[var(--bg-base)]'
        "
      >
        <p class="whitespace-pre-wrap">{{ msg.content }}</p>

        <!-- Blind spots from AI -->
        <div v-if="msg.blind_spots?.length" class="mt-2 pt-2 border-t border-[var(--border-default)]/50">
          <div v-for="(spot, j) in msg.blind_spots" :key="j" class="flex items-center gap-1.5 mb-1">
            <span
              class="px-1.5 py-0.5 text-[9px] font-semibold rounded bg-[var(--color-error)]/20 text-[var(--color-error)]"
            >
              {{ spot.severity }}
            </span>
            <span class="text-[11px] text-[var(--accent-purple)]">{{ spot.tag }}</span>
          </div>
        </div>

        <!-- Follow-up question -->
        <p
          v-if="msg.follow_up"
          class="mt-2 text-xs text-[var(--text-muted)] italic border-t border-[var(--border-default)]/50 pt-2"
        >
          {{ msg.follow_up }}
        </p>
      </div>
    </div>

    <!-- User reply input -->
    <div v-if="active && !completed" class="flex gap-2">
      <input
        v-model="reply"
        class="flex-1 bg-[var(--bg-base)] border border-[var(--border-default)] rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] placeholder-[#585b70] focus:outline-none focus:border-[var(--border-focus)] transition-colors"
        placeholder="Type your reply..."
        @keydown.enter="handleSubmit"
      />
      <button
        class="px-3 py-2 rounded-lg bg-[var(--accent-purple)] text-[var(--bg-base)] text-sm font-medium hover:bg-[var(--accent-lavender)] transition-colors disabled:opacity-50"
        :disabled="!reply.trim() || submitting"
        @click="handleSubmit"
      >
        {{ submitting ? '...' : t('quiz.submit') }}
      </button>
    </div>

    <!-- End diagnosis button -->
    <button
      v-if="active && !completed && messages.length > 0"
      class="w-full py-1.5 rounded-md text-xs text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-elevated)] transition-colors"
      @click="$emit('endDiagnosis')"
    >
      End Diagnosis & View Report
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from '@/composables/useI18n';
import type { DiagnosisRound } from '@/types/diagnosis';

const { t } = useI18n();

const props = defineProps<{
  messages: DiagnosisRound[];
  active: boolean;
  completed: boolean;
  submitting?: boolean;
}>();

const emit = defineEmits<{
  sendReply: [text: string];
  endDiagnosis: [];
}>();

const reply = ref('');

function handleSubmit() {
  const text = reply.value.trim();
  if (!text) return;
  emit('sendReply', text);
  reply.value = '';
}
</script>
