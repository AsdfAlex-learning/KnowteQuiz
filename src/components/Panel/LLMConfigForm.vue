<template>
  <div class="space-y-4">
    <h3 class="text-xs font-semibold uppercase tracking-wider text-[var(--text-muted)]">
      {{ t('settings_page.llm_config') }}
    </h3>

    <div class="space-y-3">
      <label class="block">
        <span class="text-xs text-[var(--text-muted)] mb-1 block">
          {{ t('settings_page.provider') }}
        </span>
        <select
          :value="selectedProviderId"
          class="w-full bg-[var(--bg-base)] border border-[var(--border-default)] rounded-md px-3 py-1.5 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--border-focus)] transition-colors"
          @change="handleProviderChange"
        >
          <option value="custom">{{ t('settings_page.provider_custom') }}</option>
          <option v-for="preset in LLM_PROVIDER_PRESETS" :key="preset.id" :value="preset.id">
            {{ preset.label }}
          </option>
        </select>
        <p class="mt-1 text-[11px] text-[var(--text-faint)]">
          {{ t('settings_page.provider_hint') }}
        </p>
      </label>

      <label class="block">
        <span class="text-xs text-[var(--text-muted)] mb-1 block">{{ t('settings_page.api_url') }}</span>
        <input
          :value="llm.base_url"
          class="w-full bg-[var(--bg-base)] border border-[var(--border-default)] rounded-md px-3 py-1.5 text-sm text-[var(--text-primary)] placeholder-[var(--text-faint)] focus:outline-none focus:border-[var(--border-focus)] transition-colors font-mono"
          placeholder="https://api.example.com/v1"
          @input="update('base_url', ($event.target as HTMLInputElement).value)"
        />
      </label>

      <label class="block">
        <span class="text-xs text-[var(--text-muted)] mb-1 block">{{ t('settings_page.api_key') }}</span>
        <input
          :value="llm.api_key"
          type="password"
          class="w-full bg-[var(--bg-base)] border border-[var(--border-default)] rounded-md px-3 py-1.5 text-sm text-[var(--text-primary)] placeholder-[var(--text-faint)] focus:outline-none focus:border-[var(--border-focus)] transition-colors"
          placeholder="sk-..."
          @input="update('api_key', ($event.target as HTMLInputElement).value)"
        />
        <p v-if="selectedPreset && !selectedPreset.requiresApiKey" class="mt-1 text-[11px] text-[var(--text-faint)]">
          {{ t('settings_page.api_key_optional') }}
        </p>
      </label>

      <div class="block">
        <span class="text-xs text-[var(--text-muted)] mb-1 block">{{ t('settings_page.model') }}</span>
        <div class="flex items-center gap-2">
          <input
            :value="llm.model"
            list="llm-model-options"
            class="flex-1 min-w-0 bg-[var(--bg-base)] border border-[var(--border-default)] rounded-md px-3 py-1.5 text-sm text-[var(--text-primary)] placeholder-[var(--text-faint)] focus:outline-none focus:border-[var(--border-focus)] transition-colors font-mono"
            placeholder="model-name"
            @input="update('model', ($event.target as HTMLInputElement).value)"
          />
          <datalist id="llm-model-options">
            <option v-for="model in models" :key="model" :value="model" />
          </datalist>
          <button
            class="shrink-0 px-3 py-1.5 rounded-md text-xs font-medium bg-[var(--bg-elevated)] text-[var(--text-primary)] border border-[var(--border-default)] hover:bg-[var(--bg-active)] transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            :disabled="fetching || !llm.base_url.trim()"
            @click="fetchModels"
          >
            {{ fetching ? t('settings_page.fetching_models') : t('settings_page.fetch_models') }}
          </button>
        </div>
        <p
          v-if="modelsMessage"
          class="mt-1 text-[11px]"
          :class="modelsError ? 'text-[var(--color-error)]' : 'text-[var(--text-faint)]'"
        >
          {{ modelsMessage }}
        </p>
      </div>

      <div class="grid grid-cols-2 gap-3">
        <label class="block">
          <span class="text-xs text-[var(--text-muted)] mb-1 block">{{ t('settings_page.temperature') }}</span>
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
          <span class="text-xs text-[var(--text-muted)] mb-1 block">{{ t('settings_page.max_tokens') }}</span>
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
import { computed, ref } from 'vue';
import { useI18n } from '@/composables/useI18n';
import { listLlmModels } from '@/services/settings';
import { LLM_PROVIDER_PRESETS, findProviderPreset } from '@/utils/llmProviders';
import type { SettingsLLM } from '@/types/settings';

const { t } = useI18n();

const props = defineProps<{
  llm: SettingsLLM;
}>();

const emit = defineEmits<{
  'update:llm': [value: SettingsLLM];
}>();

const models = ref<string[]>([]);
const fetching = ref(false);
const modelsMessage = ref<string | null>(null);
const modelsError = ref(false);

const selectedPreset = computed(() => findProviderPreset(props.llm.provider));
const selectedProviderId = computed(() => selectedPreset.value?.id ?? 'custom');

function update(key: keyof SettingsLLM, value: string | number) {
  emit('update:llm', { ...props.llm, [key]: value });
}

function handleProviderChange(event: Event) {
  const id = (event.target as HTMLSelectElement).value;
  const preset = findProviderPreset(id);
  models.value = [];
  modelsMessage.value = null;
  modelsError.value = false;

  if (!preset) {
    emit('update:llm', { ...props.llm, provider: 'custom' });
    return;
  }

  emit('update:llm', {
    ...props.llm,
    provider: preset.id,
    base_url: preset.baseUrl,
    ...(preset.defaultModel ? { model: preset.defaultModel } : {}),
  });
}

async function fetchModels() {
  if (!props.llm.base_url.trim() || fetching.value) return;
  fetching.value = true;
  modelsMessage.value = null;
  modelsError.value = false;
  try {
    const list = await listLlmModels(props.llm);
    models.value = list;
    if (list.length === 0) {
      modelsMessage.value = t('settings_page.models_empty');
    } else {
      modelsMessage.value = t('settings_page.models_found', { count: list.length });
      if (!props.llm.model && list.length > 0) {
        emit('update:llm', { ...props.llm, model: list[0] });
      }
    }
  } catch (error) {
    modelsError.value = true;
    modelsMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    fetching.value = false;
  }
}
</script>
