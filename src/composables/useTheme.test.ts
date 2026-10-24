import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { nextTick } from 'vue';
import { useTheme } from './useTheme';
import { useSettingsStore } from '@/stores/settings';
import { defaultSettings } from '@/utils/defaults';

describe('useTheme', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    document.body.removeAttribute('data-glassmorphism');
  });

  function prepare() {
    const settingsStore = useSettingsStore();
    settingsStore.settings.theme_config = defaultSettings().theme_config;
    useTheme();
    return settingsStore;
  }

  it('re-applies the theme when appearance fields change', async () => {
    const settingsStore = prepare();
    await nextTick();

    settingsStore.settings.theme_config.background_color = '#101014';
    await nextTick();

    expect(document.documentElement.style.getPropertyValue('--bg-base')).toBe('#101014');
  });

  it('does not re-apply the theme when video playback state updates', async () => {
    const setProperty = vi.spyOn(document.documentElement.style, 'setProperty');
    const settingsStore = prepare();
    settingsStore.settings.theme_config.background_video = '/media/lecture.mp4';
    settingsStore.settings.theme_config.video_playing = true;
    await nextTick();
    setProperty.mockClear();

    settingsStore.settings.theme_config.video_time = 1.5;
    await nextTick();
    settingsStore.settings.theme_config.video_time = 2.5;
    await nextTick();
    settingsStore.settings.theme_config.video_playing = false;
    await nextTick();

    expect(setProperty).not.toHaveBeenCalled();
  });

  it('sets colourless glass tokens for dark and light presets', async () => {
    const settingsStore = prepare();
    await nextTick();
    const style = document.documentElement.style;

    expect(style.getPropertyValue('--glass-rgb')).toBe('255, 255, 255');
    expect(style.getPropertyValue('--glass-border')).toBe('rgba(255, 255, 255, 0.15)');
    expect(style.getPropertyValue('--glass-shadow')).toContain('rgba(3, 7, 18');

    settingsStore.settings.theme_config.preset = 'vscode-light-plus';
    await nextTick();

    expect(style.getPropertyValue('--glass-rgb')).toBe('15, 23, 42');
    expect(style.getPropertyValue('--glass-border')).toBe('rgba(15, 23, 42, 0.10)');
    expect(style.getPropertyValue('--glass-shadow')).toContain('rgba(15, 23, 42');
  });
});
