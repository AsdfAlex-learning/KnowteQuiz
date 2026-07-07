<template>
  <div class="space-y-4">
    <h3 class="text-xs font-semibold uppercase tracking-wider text-[var(--text-muted)]">
      LLM Configuration
    </h3>

    <div class="space-y-3">
      <label class="block">
        <span class="text-xs text-[var(--text-muted)] mb-1 block">Provider</span>
        <input
          :value="llm.provider"
          class="w-full bg-[var(--bg-base)] border border-[var(--border-default)] rounded-md px-3 py-1.5 text-sm text-[var(--text-primary)] placeholder-[var(--text-faint)] focus:outline-none focus:border-[var(--border-focus)] transition-colors"
          placeholder="openai-compatible"
          @input="update('provider', ($event.target as HTMLInputElement).value)"
        />
      </label>

      <label class="block">
        <span class="text-xs text-[var(--text-muted)] mb-1 block">API URL</span>
        <input
          :value="llm.base_url"
          class="w-full bg-[var(--bg-base)] border border-[var(--border-default)] rounded-md px-3 py-1.5 text-sm text-[var(--text-primary)] placeholder-[var(--text-faint)] focus:outline-none focus:border-[var(--border-focus)] transition-colors font-mono"
          placeholder="http://localhost:11434/v1"
          @input="update('base_url', ($event.target as HTMLInputElement).value)"
        />
      </label>

      <label class="block">
        <span class="text-xs text-[var(--text-muted)] mb-1 block">API Key</span>
        <input
          :value="llm.api_key"
          type="password"
          class="w-full bg-[var(--bg-base)] border border-[var(--border-default)] rounded-md px-3 py-1.5 text-sm text-[var(--text-primary)] placeholder-[var(--text-faint)] focus:outline-none focus:border-[var(--border-focus)] transition-colors"
          placeholder="sk-..."
          @input="update('api_key', ($event.target as HTMLInputElement).value)"
        />
      </label>

      <label class="block">
        <span class="text-xs text-[var(--text-muted)] mb-1 block">Model</span>
        <input
          :value="llm.model"
          class="w-full bg-[var(--bg-base)] border border-[var(--border-default)] rounded-md px-3 py-1.5 text-sm text-[var(--text-primary)] placeholder-[var(--text-faint)] focus:outline-none focus:border-[var(--border-focus)] transition-colors font-mono"
          placeholder="qwen2.5:7b"
          @input="update('model', ($event.target as HTMLInputElement).value)"
        />
      </label>

      <div class="grid grid-cols-2 gap-3">
        <label class="block">
          <span class="text-xs text-[var(--text-muted)] mb-1 block">Temperature</span>
          <input
            :value="llm.temperature"
            type="number"
            step="0.1"
            min="0"
            max="2"
            class="w-full bg-[var(--bg-base)] border border-[var(--border-default)] rounded-md px-3 py-1.5 text-sm text-[var(--text-primary)] placeholder-[var(--text-faint)] focus:outline-none focus:border-[var(--border-focus)] transition-colors"
            placeholder="0.7"
            @input="update('temperature', Number(($event.target as HTMLInputElement).value))"
          />
        </label>

        <label class="block">
          <span class="text-xs text-[var(--text-muted)] mb-1 block">Max Tokens</span>
          <input
            :value="llm.max_tokens"
            type="number"
            min="1"
            max="128000"
            class="w-full bg-[var(--bg-base)] border border-[var(--border-default)] rounded-md px-3 py-1.5 text-sm text-[var(--text-primary)] placeholder-[var(--text-faint)] focus:outline-none focus:border-[var(--border-focus)] transition-colors"
            placeholder="4096"
            @input="update('max_tokens', Number(($event.target as HTMLInputElement).value))"
          />
        </label>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { SettingsLLM } from '@/types/settings'

const props = defineProps<{
  llm: SettingsLLM
}>()

const emit = defineEmits<{
  'update:llm': [value: SettingsLLM]
}>()

function update(key: keyof SettingsLLM, value: string | number) {
  emit('update:llm', { ...props.llm, [key]: value })
}
</script>
