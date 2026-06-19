<template>
  <div class="space-y-4">
    <h3 class="text-xs font-semibold uppercase tracking-wider text-[var(--text-muted)]">
      Quiz Defaults
    </h3>

    <div class="space-y-3">
      <label class="block">
        <span class="text-xs text-[var(--text-muted)] mb-1 block">Prompt Template</span>
        <select
          :value="modelValue.prompt_template"
          class="w-full bg-[var(--bg-base)] border border-[var(--border-default)] rounded-md px-3 py-1.5 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--border-focus)] transition-colors"
          @change="update('prompt_template', ($event.target as HTMLSelectElement).value)"
        >
          <option
            v-for="tmpl in templates"
            :key="tmpl.name"
            :value="tmpl.name"
            :title="tmpl.description"
          >
            {{ tmpl.label }}
          </option>
        </select>
        <span v-if="selectedTemplate" class="text-[10px] text-[var(--text-muted)] mt-1 block">{{ selectedTemplate.description }}</span>
      </label>

      <div>
        <span class="text-xs text-[var(--text-muted)] mb-1.5 block">Question Types</span>
        <div class="flex flex-wrap gap-2">
          <button
            v-for="qt in questionTypes"
            :key="qt.value"
            class="px-2.5 py-1 text-xs rounded-md border transition-colors"
            :class="
              modelValue.default_types.includes(qt.value)
                ? 'bg-[var(--accent-purple)]/20 border-[var(--border-focus)] text-[var(--accent-purple)]'
                : 'bg-[var(--bg-base)] border-[var(--border-default)] text-[var(--text-muted)] hover:border-[var(--text-faint)]'
            "
            @click="toggleType(qt.value)"
          >
            {{ qt.label }}
          </button>
        </div>
      </div>

      <label class="block">
        <span class="text-xs text-[var(--text-muted)] mb-1 flex justify-between">
          Count
          <span class="text-[var(--text-primary)]">{{ modelValue.default_count }}</span>
        </span>
        <input
          :value="modelValue.default_count"
          type="range"
          min="1"
          max="20"
          step="1"
          class="w-full accent-[var(--accent-purple)]"
          @input="update('default_count', Number(($event.target as HTMLInputElement).value))"
        />
      </label>

      <label class="block">
        <span class="text-xs text-[var(--text-muted)] mb-1 block">Difficulty</span>
        <select
          :value="modelValue.default_difficulty"
          class="w-full bg-[var(--bg-base)] border border-[var(--border-default)] rounded-md px-3 py-1.5 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--border-focus)] transition-colors"
          @change="update('default_difficulty', ($event.target as HTMLSelectElement).value as QuizDifficulty)"
        >
          <option value="easy">Easy</option>
          <option value="medium">Medium</option>
          <option value="hard">Hard</option>
        </select>
      </label>

      <label class="block">
        <span class="text-xs text-[var(--text-muted)] mb-1 block">Language</span>
        <select
          :value="modelValue.default_language"
          class="w-full bg-[var(--bg-base)] border border-[var(--border-default)] rounded-md px-3 py-1.5 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--border-focus)] transition-colors"
          @change="update('default_language', ($event.target as HTMLSelectElement).value)"
        >
          <option value="zh">中文</option>
          <option value="en">English</option>
        </select>
      </label>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { SettingsQuiz } from '@/types/settings'
import { ref, computed, onMounted } from 'vue'
import type { QuestionType, QuizDifficulty } from '@/types/quiz'
import { listPromptTemplates } from '@/services/mistake'

const props = defineProps<{
  modelValue: SettingsQuiz
}>()

const emit = defineEmits<{
  'update:modelValue': [value: SettingsQuiz]
}>()


const templates = ref<Array<{ name: string; label: string; description: string }>>([])

const selectedTemplate = computed(() => {
  return templates.value.find(t => t.name === props.modelValue.prompt_template)
})

const questionTypes: { value: QuestionType; label: string }[] = [
  { value: 'single', label: 'Single Choice' },
  { value: 'short', label: 'Short Answer' },
]

function toggleType(type: QuestionType) {
  const current = [...props.modelValue.default_types]
  const idx = current.indexOf(type)
  if (idx >= 0) {
    current.splice(idx, 1)
  } else {
    current.push(type)
  }
  if (current.length > 0) {
    emit('update:modelValue', { ...props.modelValue, default_types: current })
  }
}

function update<K extends keyof SettingsQuiz>(key: K, value: SettingsQuiz[K]) {
  emit('update:modelValue', { ...props.modelValue, [key]: value })
}

onMounted(async () => {
  try {
    templates.value = await listPromptTemplates()
  } catch (e) {
    console.error('Failed to load prompt templates:', e)
    templates.value = [
      { name: 'default', label: 'Default', description: 'Balanced, comprehensive question generation' },
      { name: 'creative', label: 'Creative', description: 'Generates open-ended, scenario-based questions' },
      { name: 'strict', label: 'Strict', description: 'Precise, rigorous academic assessment' },
    ]
  }
})
</script>
