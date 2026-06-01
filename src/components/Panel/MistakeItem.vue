<template>
  <button
    class="w-full text-left px-3 py-2.5 rounded-lg transition-colors group"
    :class="
      active
        ? 'bg-[#45475a]/60'
        : 'hover:bg-[#313244]'
    "
    @click="$emit('select', mistake.id)"
  >
    <div class="flex items-start justify-between gap-2">
      <p class="text-sm text-[#f8f8f2] leading-snug line-clamp-2 flex-1">
        {{ mistake.question }}
      </p>
      <span
        class="shrink-0 mt-0.5 px-1.5 py-0.5 text-[10px] font-semibold uppercase rounded"
        :class="
          mistake.mode === 'advanced'
            ? 'bg-[#cba6f7]/20 text-[#cba6f7]'
            : 'bg-[#89b4fa]/20 text-[#89b4fa]'
        "
      >
        {{ mistake.mode }}
      </span>
    </div>
    <p class="text-[11px] text-[#585b70] mt-1">
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
