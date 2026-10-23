import { describe, expect, it } from 'vitest';
import { LLM_PROVIDER_PRESETS, findProviderPreset } from './llmProviders';

describe('LLM provider presets', () => {
  it('has unique ids and OpenAI-compatible base URLs', () => {
    const ids = LLM_PROVIDER_PRESETS.map((preset) => preset.id);
    expect(new Set(ids).size).toBe(ids.length);
    for (const preset of LLM_PROVIDER_PRESETS) {
      expect(preset.baseUrl).toMatch(/^https?:\/\//);
      expect(preset.label.trim()).not.toBe('');
    }
  });

  it('marks local providers as not requiring an API key', () => {
    expect(findProviderPreset('ollama')?.requiresApiKey).toBe(false);
    expect(findProviderPreset('lmstudio')?.requiresApiKey).toBe(false);
    expect(findProviderPreset('openai')?.requiresApiKey).toBe(true);
  });

  it('returns undefined for custom or unknown providers', () => {
    expect(findProviderPreset('custom')).toBeUndefined();
    expect(findProviderPreset('openai-compatible')).toBeUndefined();
    expect(findProviderPreset(undefined)).toBeUndefined();
  });
});
