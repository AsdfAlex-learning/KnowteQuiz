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
            ? 'bg-[#313244] text-[#cdd6f4]'
            : 'bg-[#cba6f7] text-[#1e1e2e]'
        "
      >
        <p class="whitespace-pre-wrap">{{ msg.content }}</p>

        <!-- Blind spots from AI -->
        <div v-if="msg.blind_spots?.length" class="mt-2 pt-2 border-t border-[#45475a]/50">
          <div
            v-for="(spot, j) in msg.blind_spots"
            :key="j"
            class="flex items-center gap-1.5 mb-1"
          >
            <span class="px-1.5 py-0.5 text-[9px] font-semibold rounded bg-[#f38ba8]/20 text-[#f38ba8]">
              {{ spot.severity }}
            </span>
            <span class="text-[11px] text-[#cba6f7]">{{ spot.tag }}</span>
          </div>
        </div>

        <!-- Follow-up question -->
        <p
          v-if="msg.follow_up"
          class="mt-2 text-xs text-[#a6adc8] italic border-t border-[#45475a]/50 pt-2"
        >
          {{ msg.follow_up }}
        </p>
      </div>
    </div>

    <!-- User reply input -->
    <div v-if="active && !completed" class="flex gap-2">
      <input
        v-model="reply"
        class="flex-1 bg-[#1e1e2e] border border-[#45475a] rounded-lg px-3 py-2 text-sm text-[#f8f8f2] placeholder-[#585b70] focus:outline-none focus:border-[#cba6f7] transition-colors"
        placeholder="Type your reply..."
        @keydown.enter="handleSubmit"
      />
      <button
        class="px-3 py-2 rounded-lg bg-[#cba6f7] text-[#1e1e2e] text-sm font-medium hover:bg-[#b4befe] transition-colors disabled:opacity-50"
        :disabled="!reply.trim() || submitting"
        @click="handleSubmit"
      >
        {{ submitting ? '...' : 'Send' }}
      </button>
    </div>

    <!-- End diagnosis button -->
    <button
      v-if="active && !completed && messages.length > 0"
      class="w-full py-1.5 rounded-md text-xs text-[#a6adc8] hover:text-[#f8f8f2] hover:bg-[#313244] transition-colors"
      @click="$emit('endDiagnosis')"
    >
      End Diagnosis & View Report
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import type { DiagnosisRound } from '@/types/diagnosis'

const props = defineProps<{
  messages: DiagnosisRound[]
  active: boolean
  completed: boolean
  submitting?: boolean
}>()

const emit = defineEmits<{
  sendReply: [text: string]
  endDiagnosis: []
}>()

const reply = ref('')

function handleSubmit() {
  const text = reply.value.trim()
  if (!text) return
  emit('sendReply', text)
  reply.value = ''
}
</script>
