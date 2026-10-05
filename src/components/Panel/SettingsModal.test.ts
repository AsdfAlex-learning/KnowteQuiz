import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import SettingsModal from './SettingsModal.vue';
import { defaultSettings } from '@/utils/defaults';
import * as settingsService from '@/services/settings';

vi.mock('@/services/settings', () => ({
  getSettings: vi.fn(),
  saveSettings: vi.fn(),
  testConnection: vi.fn(),
  backupData: vi.fn(),
  getDataStatus: vi.fn(),
  restoreLatestBackup: vi.fn(),
  openDataDir: vi.fn(),
  probeLlm: vi.fn(),
  saveMediaBytes: vi.fn(),
}));

vi.mock('@/services/quiz', () => ({
  cleanupSessions: vi.fn(),
}));

vi.mock('@/services/mistake', () => ({
  listPromptTemplates: vi.fn(async () => []),
}));

// The modal teleports to <body>, so query the document rather than the wrapper.
function mountModal() {
  return mount(SettingsModal, {
    props: { modelValue: true },
    attachTo: document.body,
    global: { plugins: [createPinia()] },
  });
}

async function clickButton(text: string) {
  const button = Array.from(document.body.querySelectorAll('button')).find(
    (candidate) => candidate.textContent?.trim() === text
  );
  if (!button) throw new Error(`button not found: ${text}`);
  button.click();
  await vi.dynamicImportSettled();
}

function bodyText(): string {
  return document.body.textContent ?? '';
}

describe('SettingsModal', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    vi.mocked(settingsService.getSettings).mockResolvedValue(defaultSettings());
    vi.mocked(settingsService.getDataStatus).mockResolvedValue({
      data_dir: 'C:/Users/Alex/AppData/Roaming/knowtequiz',
      files: [],
    });
    vi.stubGlobal(
      'confirm',
      vi.fn(() => true)
    );
  });

  afterEach(() => {
    document.body.innerHTML = '';
    vi.unstubAllGlobals();
  });

  it('renders failed connection tests with the error state', async () => {
    vi.mocked(settingsService.testConnection).mockResolvedValue({
      ok: false,
      kind: 'auth',
      message: 'LLM endpoint rejected the API key',
      status: 401,
    });
    mountModal();

    await clickButton('Test Connection');

    expect(bodyText()).toContain('LLM endpoint rejected the API key');
    expect(document.body.querySelector('.bg-\\[var\\(--color-error\\)\\]\\/10')).not.toBeNull();
  });

  it('probes the edited llm config without reading it from saved settings', async () => {
    vi.mocked(settingsService.testConnection).mockResolvedValue({
      ok: true,
      kind: 'ok',
      message: 'Connection successful',
      status: 200,
    });
    mountModal();

    await clickButton('Test Connection');

    expect(settingsService.testConnection).toHaveBeenCalledOnce();
    const probedLlm = vi.mocked(settingsService.testConnection).mock.calls[0][0] as {
      base_url?: string;
    };
    // The probe receives the buffered local LLM config object.
    expect(probedLlm).toBeTypeOf('object');
    expect(probedLlm.base_url).toBeTypeOf('string');
  });

  it('shows settings persistence errors after saving fails', async () => {
    vi.mocked(settingsService.saveSettings).mockRejectedValue(new Error('HTTP 500: Failed to write settings.json'));
    mountModal();

    await clickButton('Save Settings');

    expect(bodyText()).toContain('Failed to write settings.json');
  });

  it('shows data backup results after a manual backup succeeds', async () => {
    vi.mocked(settingsService.backupData).mockResolvedValue({
      backup_dir: 'C:/Users/Alex/AppData/Roaming/knowtequiz/backups/20260621-120000',
      files: ['settings.json', 'mistakes.json'],
    });
    const wrapper = mountModal();

    await clickButton('Backup Data Now');

    expect(bodyText()).toContain('Backed up 2 files');
    expect(bodyText()).toContain('20260621-120000');
    wrapper.unmount();
  });

  it('shows restore results after restoring the latest backup', async () => {
    vi.mocked(settingsService.restoreLatestBackup).mockResolvedValue({
      backup_dir: 'C:/Users/Alex/AppData/Roaming/knowtequiz/backups/20260621-120000',
      pre_restore_backup_dir: 'C:/Users/Alex/AppData/Roaming/knowtequiz/backups/20260621-130000',
      files: ['settings.json', 'mistakes.json'],
    });
    const wrapper = mountModal();

    await clickButton('Restore Latest Backup');

    expect(bodyText()).toContain('Restored 2 files');
    expect(bodyText()).toContain('20260621-120000');
    wrapper.unmount();
  });

  it('shows data file sizes and modified times from the settings store', async () => {
    vi.mocked(settingsService.getDataStatus).mockResolvedValue({
      data_dir: 'C:/Users/Alex/AppData/Roaming/knowtequiz',
      files: [
        {
          name: 'settings.json',
          exists: true,
          size_bytes: 2048,
          modified_at: '2026-06-21T12:00:00Z',
        },
        {
          name: 'mistakes.json',
          exists: false,
          size_bytes: 0,
          modified_at: null,
        },
      ],
    });
    const wrapper = mountModal();
    await vi.dynamicImportSettled();

    expect(bodyText()).toContain('Data Files');
    expect(bodyText()).toContain('settings.json');
    expect(bodyText()).toContain('2 KB');
    expect(bodyText()).toContain('2026-06-21 12:00');
    expect(bodyText()).toContain('mistakes.json');
    expect(bodyText()).toContain('Missing');
    wrapper.unmount();
  });

  it('closes on escape and focuses inside the dialog with dialog semantics', async () => {
    const wrapper = mountModal();
    await vi.dynamicImportSettled();

    const dialog = document.body.querySelector<HTMLElement>('[role="dialog"]');
    expect(dialog).not.toBeNull();
    expect(dialog?.getAttribute('aria-modal')).toBe('true');
    expect(dialog?.getAttribute('aria-labelledby')).toBe('settings-modal-title');
    expect(dialog?.contains(document.activeElement)).toBe(true);

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));

    const emitted = wrapper.emitted('update:modelValue');
    expect(emitted?.[emitted.length - 1]).toEqual([false]);
  });
});
