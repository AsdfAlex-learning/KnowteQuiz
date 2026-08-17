import { computed, type ComputedRef } from 'vue';
import { useSettingsStore } from '@/stores/settings';
import zhCN from '@/locales/zh-CN.json';
import en from '@/locales/en.json';
import ja from '@/locales/ja.json';
import ko from '@/locales/ko.json';

const messages: Record<string, Record<string, unknown>> = {
  'zh-CN': zhCN,
  en,
  ja,
  ko,
};

function getNestedValue(obj: Record<string, unknown>, path: string): string | undefined {
  const keys = path.split('.');
  let current: unknown = obj;
  for (const key of keys) {
    if (current === null || current === undefined || typeof current !== 'object') {
      return undefined;
    }
    current = (current as Record<string, unknown>)[key];
  }
  return typeof current === 'string' ? current : undefined;
}

export type TranslationKey = string;

export function useI18n(): {
  t: (key: TranslationKey, params?: Record<string, string | number>) => string;
  locale: ComputedRef<string>;
  availableLocales: { value: string; label: string }[];
  setLocale: (locale: string) => void;
} {
  const settingsStore = useSettingsStore();

  const locale = computed(() => settingsStore.settings.ui_language || 'zh-CN');

  const availableLocales = [
    { value: 'zh-CN', label: '简体中文' },
    { value: 'en', label: 'English' },
    { value: 'ja', label: '日本語' },
    { value: 'ko', label: '한국어' },
  ];

  function t(key: TranslationKey, params?: Record<string, string | number>): string {
    const lang = locale.value;
    const langMessages = messages[lang] || messages['zh-CN'];
    let text = getNestedValue(langMessages as Record<string, unknown>, key);

    // Fallback to zh-CN
    if (text === undefined) {
      text = getNestedValue(messages['zh-CN'] as Record<string, unknown>, key);
    }

    // Fallback to key itself
    if (text === undefined) {
      return key;
    }

    // Replace {param} placeholders
    if (params) {
      for (const [paramKey, paramValue] of Object.entries(params)) {
        text = text.replace(new RegExp(`\\{${paramKey}\\}`, 'g'), String(paramValue));
      }
    }

    return text;
  }

  function setLocale(newLocale: string) {
    settingsStore.settings.ui_language = newLocale;
    settingsStore.persistSettings();
  }

  return { t, locale, availableLocales, setLocale };
}
