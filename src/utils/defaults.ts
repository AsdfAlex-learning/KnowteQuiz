import type { Settings } from '../types/settings';

export function defaultSettings(): Settings {
  return {
    version: '1.0.0',
    theme: 'obsidian-dark',
    ui_language: 'zh-CN',
    llm: {
      provider: 'openai-compatible',
      base_url: 'http://localhost:11434/v1',
      api_key: '',
      model: 'qwen2.5:7b',
      max_tokens: 4096,
      temperature: 0.7,
    },
    ui: {
      layout: {
        left_visible: true,
        right_visible: true,
        left_width: 280,
        right_width: 360,
      },
    },
    quiz: {
      default_types: ['single', 'short'],
      default_language: 'zh',
      default_count: 5,
      default_mode: 'basic',
      default_difficulty: 'medium',
      prompt_template: 'default',
      advanced: {
        max_diagnosis_rounds: 3,
        require_reasoning: true,
        show_diagnosis_report: true,
      },
    },
    workspace: {
      root_path: null,
      expanded_dirs: [],
      selected_path: null,
      scroll_positions: {},
    },
  };
}
