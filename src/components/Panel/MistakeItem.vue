<template>
  <button
    class="w-full text-left px-3 py-2.5 rounded-lg transition-colors card-hover group"
    :class="
      active
        ? 'bg-[var(--bg-active)]/60'
        : 'hover:bg-[var(--bg-elevated)]'
    "
    @click="$emit('select', mistake.id)"
  >
    <div class="flex items-start justify-between gap-2">
      <p class="text-sm text-[var(--text-primary)] leading-snug line-clamp-2 flex-1">
        {{ mistake.question }}
      </p>
      <span
        class="shrink-0 mt-0.5 px-1.5 py-0.5 text-[10px] font-semibold uppercase rounded"
        :class="
          mistake.mode === 'advanced'
            ? 'bg-[var(--accent-purple)]/20 text-[var(--accent-purple)]'
            : 'bg-[var(--accent-blue)]/20 text-[var(--accent-blue)]'
        "
      >
        {{ mistake.mode }}
      </span>
    </div>
    <p class="text-[11px] text-[var(--text-faint)] mt-1">
      {{ formattedDate }}
    </p>
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { MistakeEntry } from '@/types/mistake'

const props = defineProps<{
  mistake: MistakeEntry
  active: boolean
}>()

defineEmits<{
  select: [id: string]
}>()

const formattedDate = computed(() => {
  const d = new Date(props.mistake.created_at)
  return d.toLocaleDateString(undefined, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
})
</script>
