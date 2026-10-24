import type { Settings } from '../types/settings';

export function defaultSettings(): Settings {
  return {
    version: '1.0.0',
    ui_language: 'en',
    llm: {
      provider: 'custom',
      // Blank on purpose: "ready" must come from a real connection test,
      // never from a pre-filled local address.
      base_url: '',
      api_key: '',
      model: 'qwen2.5:7b',
      max_tokens: 4096,
      temperature: 0.7,
      verified_at: null,
    },
    ui: {
      layout: {
        left_visible: true,
        right_visible: true,
        left_width: 280,
        right_width: 360,
      },
      last_mode: 'reader',
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
      streak_days: 0,
      last_active_date: null,
    },
    exploration: {
      default_baseline: 50,
    },
    theme_config: {
      preset: 'vscode-dark-plus',
      background_color: '#1e1e1e',
      accent_color: '#007acc',
      background_image: null,
      glassmorphism: {
        enabled: false,
        opacity: 40,
        blur: 40,
        scope: 'global',
      },
      overlay_opacity: 0,
      background_video: null,
      video_playing: true,
      video_time: 0,
    },
    cors: {
      allow_origins: [],
    },
  };
}
